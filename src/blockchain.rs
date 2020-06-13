//! The blockchain data structure

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io::stdout;
use std::mem;
use std::time::SystemTime;
use uuid::Uuid;

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    index: u64,
    timestamp: u128,
    proof: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
}

impl Block {
    pub fn get_genesis() -> Self {
        Block {
            index: 0,
            timestamp: 0,
            proof: 100,
            transactions: Vec::new(),
            previous_hash: String::from("1"),
        }
    }

    /// Returns the index of the Block in the chain.
    pub fn get_index(&self) -> u64 {
        self.index
    }

    /// Hashes a Block.
    pub fn get_hash(&self) -> String {
        let block_string = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.input_str(&block_string);
        hasher.result_str()
    }
}

pub struct Blockchain {
    current_transactions: Vec<Transaction>,
    // blocks is non-empty
    blocks: Vec<Block>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain::new()
    }
}

impl Blockchain {
    /// Creates a new Blockchain with only the genesis block.
    pub fn new() -> Self {
        Blockchain {
            current_transactions: vec![],
            blocks: vec![Block::get_genesis()],
        }
    }

    /// Creates a blockchain from given blocks.
    pub fn from_blocks(blocks: Vec<Block>) -> Self {
        Blockchain {
            current_transactions: vec![],
            blocks,
        }
    }

    /// Returns a copy of the blocks the chain owns.
    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }

    /// Returns the number of blocks in the blockchain, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Adds a new transaction to the list of transactions.
    pub fn add_new_transaction(&mut self, transaction: &Transaction) -> bool {
        // check whether it already exists in current transactions
        for t in &self.current_transactions {
            if t.get_id() == transaction.get_id() {
                return false;
            }
        }
        // check whether it has already been added to the blockchain
        for b in &self.blocks {
            for t in &b.transactions {
                if t.get_id() == transaction.get_id() {
                    return false;
                }
            }
        }
        self.current_transactions.push(transaction.clone());
        debug!("New transaction {:?} added", transaction.id);
        true
    }

    /// Creates a new Block containing current transactions and adds it to the chain.
    pub fn create_new_block(&mut self, proof: u64, previous_hash: String) -> &Block {
        let transactions = mem::replace(&mut self.current_transactions, Vec::new());

        let block = Block {
            index: self.blocks.len() as u64,
            timestamp: get_time(),
            proof,
            transactions,
            previous_hash,
        };

        self.blocks.push(block);
        self.last_block()
    }

    /// Adds a given block to the chain. Returns `false` if the new block is invalid.
    pub fn add_new_block(&mut self, block: &Block) -> bool {
        let (block_idx, current_len) = (block.get_index(), self.blocks.len() as u64);
        match block_idx.cmp(&current_len) {
            Ordering::Less => {
                debug!("The incoming block is too old, so it is dropped");
                false
            }
            Ordering::Equal => {
                let last_block = self.last_block();
                if last_block.get_hash() != block.previous_hash
                    || !Blockchain::valid_proof(last_block.proof, block.proof)
                {
                    debug!("The incoming block is not valid");
                    false
                } else {
                    // okay, now this block looks good to us
                    // but we should check whether the block contains duplicate transactions with us
                    for t in &block.transactions {
                        let mut i = 0;
                        while i < self.current_transactions.len() {
                            if t.get_id() == self.current_transactions[i].get_id() {
                                // the order of the transactions doesn't matter, so we can swap_remove
                                self.current_transactions.swap_remove(i);
                            } else {
                                i += 1;
                            }
                        }
                    }
                    debug!("The incoming block is accepted :)");
                    self.blocks.push(block.clone());
                    true
                }
            }
            Ordering::Greater => {
                debug!("The incoming block is too new for us, we need to resolve conflicts");
                false
            }
        }
    }

    /// Returns a copy of current transactions.
    pub fn get_current_transactions(&self) -> Vec<Transaction> {
        self.current_transactions.clone()
    }

    /// Returns the last Block in the chain.
    pub fn last_block(&self) -> &Block {
        &self.blocks.last().unwrap()
    }

    /// Proof of Work algorithm.
    pub fn proof_of_work(last_proof: u64) -> u64 {
        let mut proof = 0;
        while !Blockchain::valid_proof(last_proof, proof) {
            proof += 1;
        }
        proof
    }

    /// Run PoW in the chain.
    pub fn run_pow(&self) -> u64 {
        Blockchain::proof_of_work(self.last_block().proof)
    }

    /// Validates the Proof. Does hash(last_proof, proof) contain 4 leading zeroes?
    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        let mut hasher = Sha256::new();
        hasher.input_str(&format!("{}{}", last_proof, proof));
        &hasher.result_str()[0..4] == "0000"
    }

    /// Displays the full blockchain.
    pub fn display(&self) {
        serde_json::to_writer_pretty(stdout(), &self.blocks).expect("fail to display blockchain");
    }

    /// Validates a given blockchain.
    pub fn valid_chain(chain: &Self) -> bool {
        let mut prev_block = &chain.blocks[0];
        let mut block;

        // check the genesis block
        if prev_block.proof != 100
            || !prev_block.transactions.is_empty()
            || prev_block.previous_hash != "1"
        {
            return false;
        }

        for i in 1..chain.blocks.len() {
            block = &chain.blocks[i];
            trace!("validating chain ...");
            trace!(
                "prev_block: {}",
                serde_json::to_string(&prev_block).unwrap()
            );
            trace!("block: {}", serde_json::to_string(&block).unwrap());
            trace!("");
            if prev_block.get_hash() != block.previous_hash {
                return false;
            }
            if !Blockchain::valid_proof(prev_block.proof, block.proof) {
                return false;
            }
            prev_block = block;
        }
        true
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    id: String,
    // unique identifier for one transaction
    sender: String,
    recipient: String,
    amount: i64,
}

impl Transaction {
    pub fn new(sender: &str, recipient: &str, amount: i64) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(),
            sender: sender.to_owned(),
            recipient: recipient.to_owned(),
            amount,
        }
    }

    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //    use env_logger::Env;

    #[test]
    fn test_pow() {
        assert!(Blockchain::valid_proof(100, 35293));
        assert!(Blockchain::valid_proof(35293, 35089));

        assert_eq!(Blockchain::proof_of_work(100), 35293);
        assert_eq!(Blockchain::proof_of_work(35293), 35089);
    }

    #[test]
    fn test_valid_chain() {
        //        env_logger::from_env(Env::default().default_filter_or("debug")).init();

        let mut chain = Blockchain::new();
        assert!(Blockchain::valid_chain(&chain));

        // play with the genesis block
        chain.blocks[0]
            .transactions
            .push(Transaction::new("good", "evil", 100));
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks[0].transactions.pop();
        assert!(Blockchain::valid_chain(&chain));
        chain.blocks[0].proof = 101;
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks[0].proof = 100;
        assert!(Blockchain::valid_chain(&chain));
        chain.blocks[0].previous_hash = "2".to_owned();
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks[0].previous_hash = "1".to_owned();
        assert!(Blockchain::valid_chain(&chain));

        // perform some normal operations
        chain.add_new_transaction(&Transaction::new("0", "1", 1));
        chain.add_new_transaction(&Transaction::new("1", "2", 2));
        chain.add_new_transaction(&Transaction::new("2", "3", 3));
        chain.create_new_block(chain.run_pow(), chain.last_block().get_hash());
        assert!(Blockchain::valid_chain(&chain));
        chain.create_new_block(chain.run_pow(), chain.last_block().get_hash());
        assert!(Blockchain::valid_chain(&chain));

        // tamper an intermediate block
        chain.blocks[1]
            .transactions
            .push(Transaction::new("good", "evil", 100));
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks[1].transactions.pop();
        assert!(Blockchain::valid_chain(&chain));
        let true_proof = mem::replace(&mut chain.blocks[1].proof, 123);
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks[1].proof = true_proof;
        assert!(Blockchain::valid_chain(&chain));

        // add a block without running pow
        chain.create_new_block(456, chain.last_block().get_hash());
        assert!(!Blockchain::valid_chain(&chain));
        chain.blocks.pop();
        assert!(Blockchain::valid_chain(&chain));

        // play with the genesis block again
        chain.blocks[0]
            .transactions
            .push(Transaction::new("good", "evil", 100));
        assert!(!Blockchain::valid_chain(&chain));
    }
}
