//! The blockchain data structure

use std::time::SystemTime;
use std::mem;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use crypto::digest::Digest;


#[derive(Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: SystemTime,
    proof: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
}

pub struct Blockchain {
    current_transactions: Vec<Transaction>,
    // blocks is non-empty
    blocks: Vec<Block>,
}

impl Blockchain {
    /// Creates a new Blockchain node
    pub fn new() -> Self { unimplemented!() }

    /// Creates a new Block and adds it to the chain
    pub fn new_block(&mut self, proof: u64, previous_hash: String) {
        let transactions = mem::replace(&mut self.current_transactions, Vec::new());

        let block = Block {
            index: (self.blocks.len() + 1) as u64,
            timestamp: SystemTime::now(),
            proof,
            transactions,
            previous_hash,
        };

        self.blocks.push(block);
    }

    /// Adds a new transaction to the list of transactions.
    /// Returns the index of the Block that will hold this transaction
    pub fn new_transaction(&mut self, sender: String, recipient: String, amount: i64) -> u64 {
        self.current_transactions.push(Transaction { sender, recipient, amount });
        (self.blocks.len() + 1) as u64
    }

    /// Hashes a Block
    fn hash(block: &Block) -> String {
        let block_string = serde_json::to_string(block).unwrap();
        let mut hasher = Sha256::new();
        hasher.input_str(&block_string);
        hasher.result_str()
    }

    /// Returns the last Block in the chain
    fn last_block(&self) -> &Block {
        &self.blocks.last().unwrap()
    }

    fn proof_of_work(last_proof: u64) -> u64 {
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