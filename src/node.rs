//! The blockchain node
//! TODO: Now the nodes should get synced manually. Consider adding auto message broadcasting mechanism
use crate::message::{Request, Response};
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::collections::HashSet;
use uuid::Uuid;

// self introduction for others to contact you
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct PeerInfo {
    id: String,
    address: String,
}

impl PeerInfo {
    pub fn new(address: String) -> Self {
        PeerInfo {
            id: Uuid::new_v4().to_string(),
            address,
        }
    }

    pub fn get_address(&self) -> &str{
        self.address.as_str()
    }
}

pub struct Node {
    basic_info: PeerInfo,
    chain: Blockchain,
    peers: HashSet<PeerInfo>,
}

impl Node {
    pub fn new(addr: String) -> Self {
        Node {
            basic_info: PeerInfo::new(addr),
            chain: Blockchain::new(),
            peers: HashSet::new(),
        }
    }

    pub fn get_basic_info(&self) -> PeerInfo {
        self.basic_info.clone()
    }

    /// Returns a copy of the blocks the node owns
    pub fn get_blocks(&self) -> Vec<Block> {
        self.chain.get_blocks()
    }

    /// Mines a new block
    pub fn mine(&mut self) {
        let last_block = self.chain.last_block();
        let proof = self.chain.run_pow();
        let last_hash = last_block.get_hash();
        // receive a reward for finding the proof.
        // The sender is "0" to signify that this node has mined a new coin.
        self.create_and_add_new_transaction("0", &self.basic_info.id.clone(), 1);

        let block = self.chain.new_block(proof, last_hash);
        info!(
            "[Node {}] A new block {} is forged",
            self.basic_info.id,
            block.get_index()
        );
    }

    /// Adds a new transaction
    pub fn create_and_add_new_transaction(&mut self, sender: &str, receiver: &str, amount: i64) {
        let transaction = Transaction::new(sender, receiver, amount);
        if !self.chain.add_new_transaction(&transaction){
            info!("Transaction already exists");
            return
        }
        info!(
            "[Node {}] A new transaction is added: {} -> {}, amount: {}",
            self.basic_info.id, sender, receiver, amount
        );
        self.broadcast_transaction(transaction);
    }

    // Take an incoming transaction and try to add it
    // If it already exists, drop it and do nothing
    // Else, add and broadcast it
    pub fn add_incoming_transaction(&mut self, transaction: Transaction){
        if !self.chain.add_new_transaction(&transaction){
            return
        }
        // TODO: how to avoid deadlock when broadcasting?
        // self.broadcast_transaction(transaction)
    }

    pub fn broadcast_transaction(&self, transaction: Transaction) {
        let peers = self.peers.clone();
        debug!(
            "[Node {}] broadcasts transaction {:?} to peers :{:?}",
            self.basic_info.id, transaction.get_id(), peers
        );
        for peer in peers.iter() {
            debug!("Connecting {:?}", peer);
            let socket_address = peer.get_address().to_socket_addrs().unwrap().as_slice()[0];
            match TcpStream::connect(socket_address) {
                Ok(stream) => {
                    let _ = self.send_transaction(stream, transaction.clone());
                }
                Err(e) => debug!("Connection to {:?} failed: {}", peer, e)
            }
        }
    }

    pub fn send_transaction(&self, mut stream: TcpStream, transaction: Transaction) -> Result<bool>{
        serde_json::to_writer(
            stream.try_clone()?,
            &Request::NewTransaction(self.basic_info.clone(), transaction),
        )?;
        stream.flush()?;
        debug!("Request sent");
        // There should be only one response, but we have to deserialize from a stream in this way
        for response in Deserializer::from_reader(stream.try_clone()?).into_iter::<Response>() {
            let response =
                response.map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
            return if let Response::Ack(_) = response {
                debug!("Response received");
                Ok(true)
            } else {
                Err(failure::err_msg("Invalid response received"))
            };
        }
        Err(failure::err_msg("No response"))
    }

    /// Displays the full blockchain
    pub fn display(&self) {
        self.chain.display();
    }

    /// Adds a new peer. Returns false if `addr` is not a valid socket addr
    pub fn detect_peer(&mut self, addr: &str) -> bool {
        match addr.to_socket_addrs() {
            Ok(addr) => {
                let addr = addr.as_slice();
                assert_eq!(addr.len(), 1);
                match TcpStream::connect(addr[0]){
                    Ok(stream) => {
                        if let Ok(true) = self.say_hello(stream){
                            true
                        }else{
                            false
                        }
                    }
                    Err(e) => {
                        error!("Error when communicating with {:?}: {}", addr, e);
                        false
                    }
                }
            }
            Err(_) => false,
        }
    }

    pub fn say_hello(&mut self, mut stream: TcpStream) -> Result<bool>{
        serde_json::to_writer(
            stream.try_clone()?,
            &Request::Hello(self.basic_info.clone()),
        )?;
        stream.flush()?;
        debug!("Request sent");
        // There should be only one response, but we have to deserialize from a stream in this way
        for response in Deserializer::from_reader(stream.try_clone()?).into_iter::<Response>() {
            let response =
                response.map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
            return if let Response::Ack(peer_info) = response {
                debug!("Ack for Hello received from: {:?}", peer_info);
                Ok(self.add_peer(peer_info))
            } else {
                Err(failure::err_msg("Invalid response"))
            };
        }
        return Err(failure::err_msg("No response"));
    }

    pub fn add_peer(&mut self, peer: PeerInfo) -> bool{
        if self.peers.contains(&peer){
            debug!("Peer already exists: {:?}", peer);
            false
        }else{
            debug!("New peer added: {:?}", peer);
            self.peers.insert(peer);
            true
        }
    }

    /// This is our Consensus Algorithm, it resolves conflicts
    /// by replacing our chain with the longest one in the network.
    /// Returns `true` if the chain is replaced
    pub fn resolve_conflicts(&mut self) -> bool {
        let mut ret = false;
        let peers = self.peers.clone();
        debug!(
            "[Node {}] Resolve conflict with peers :{:?}",
            self.basic_info.id, peers
        );
        for peer in peers.iter() {
            debug!("Connecting {:?}", peer);
            let socket_address = peer.get_address().to_socket_addrs().unwrap().as_slice()[0];
            match TcpStream::connect(socket_address) {
                Ok(stream) => {
                    debug!(
                        "[Node {}] Resolve conflict with peer :{:?}",
                        self.basic_info.id, peer
                    );
                    match self.resolve_conflict(stream) {
                        Ok(flag) => {
                            ret = ret || flag;
                        }
                        Err(e) => {
                            error!("Error when communicating with {:?}: {}", peer, e);
                        }
                    }
                }
                Err(e) => error!("Connection to {:?} failed: {}", peer, e),
            }
        }
        ret
    }

    fn resolve_conflict(&mut self, mut stream: TcpStream) -> Result<bool> {
        serde_json::to_writer(
            stream.try_clone()?,
            &Request::HowAreYou(self.basic_info.clone()),
        )?;
        stream.flush()?;
        debug!("Request sent");
        // There should be only one response, but we have to deserialize from a stream in this way
        for response in Deserializer::from_reader(stream.try_clone()?).into_iter::<Response>() {
            let response =
                response.map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
            return if let Response::MyBlocks(_, blocks) = response {
                debug!("Response received");
                let new_chain = Blockchain::from_blocks(blocks);
                if new_chain.len() > self.chain.len() && Blockchain::valid_chain(&new_chain) {
                    self.chain = new_chain;
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Err(failure::err_msg("Invalid response"))
            };
        }
        return Err(failure::err_msg("No response"));
    }
}
