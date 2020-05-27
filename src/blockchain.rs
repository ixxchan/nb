//! The blockchain data structure

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::SystemTime;
use std::io::stdout;

fn get_time() -> u128 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    index: u64,
    timestamp: u128,
    proof: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
}

impl Block {
    pub fn get_proof(&self) -> u64 {
        self.proof
    }

    pub fn get_index(&self) -> u64 {
        self.index
    }
    /// Hashes a Block
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

impl Blockchain {
    /// Creates a new Blockchain node
    pub fn new() -> Self {
        let mut chain = Blockchain {
            current_transactions: vec![],
            blocks: vec![],
        };
        chain.new_block(100, "1".to_owned());
        chain
    }

    /// Creates a new Block and adds it to the chain
    pub fn new_block(&mut self, proof: u64, previous_hash: String) -> &Block {
        let transactions = mem::replace(&mut self.current_transactions, Vec::new());

        let block = Block {
            index: (self.blocks.len() + 1) as u64,
            timestamp: get_time(),
            proof,
            transactions,
            previous_hash,
        };

        self.blocks.push(block);
        self.last_block()
    }

    /// Adds a new transaction to the list of transactions
    /// (which will go into the next mined block).
    /// Returns the index of the Block that will hold this transaction
    pub fn new_transaction(&mut self, sender: &str, recipient: &str, amount: i64) -> u64 {
        self.current_transactions.push(Transaction {
            sender: sender.to_owned(),
            recipient: recipient.to_owned(),
            amount,
        });
        (self.blocks.len() + 1) as u64
    }

    /// Returns the last Block in the chain
    pub fn last_block(&self) -> &Block {
        &self.blocks.last().unwrap()
    }

    /// Proof of Work algorithm
    pub fn proof_of_work(last_proof: u64) -> u64 {
        let mut proof = 0;
        while Blockchain::valid_proof(last_proof, proof) == false {
            proof += 1;
        }
        proof
    }

    /// Validates the Proof. Does hash(last_proof, proof) contain 4 leading zeroes?
    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        let mut hasher = Sha256::new();
        hasher.input_str(&format!("{}{}", last_proof, proof));
        return &hasher.result_str()[0..4] == "0000";
    }

    pub fn display(&self) {
        serde_json::to_writer_pretty(stdout(), &self.blocks).expect("fail to display blockchain");
    }
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow() {
        assert!(Blockchain::valid_proof(100, 35293));
        assert!(Blockchain::valid_proof(35293, 35089));

        assert_eq!(Blockchain::proof_of_work(100), 35293);
        assert_eq!(Blockchain::proof_of_work(35293), 35089);
    }
}
