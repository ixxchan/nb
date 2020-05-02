//! The blockchain node
use crate::Blockchain;

struct Node {
    index: u64,
    chain: Blockchain,
}

impl Node {
    pub fn new() -> Self { unimplemented!() }

    /// Mines a new block
    pub fn mine(&mut self) {
        let last_block = self.chain.last_block();
        let proof = Blockchain::proof_of_work(last_block.get_proof());
        let last_hash = last_block.get_hash();
        // receive a reward for finding the proof.
        // The sender is "0" to signify that this node has mined a new coin.
        self.new_transaction("0", &self.index.to_string(), 1);

        let block = self.chain.new_block(proof, last_hash);
        info!("[Node {}] A new block {} is forged", self.index, block.get_index());
    }

    /// Adds a new transaction
    pub fn new_transaction(&mut self, sender: &str, receiver: &str, amount: i64) {
        self.chain.new_transaction(sender, receiver, amount);
        info!("[Node {}] A new transaction is added: {} -> {}, amount: {}", self.index, sender, receiver, amount);
    }

    /// Displays the full blockchain
    pub fn display() { unimplemented!() }
}