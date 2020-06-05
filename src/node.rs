//! The blockchain node
//! TODO: Now the nodes should get synced manually. Consider adding auto message broadcasting mechanism
use crate::message::{Request, Response};
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashSet;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
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

    pub fn get_address(&self) -> &str {
        self.address.as_str()
    }
}

/// TODO: add consensus protocol specification
pub struct Node {
    basic_info: PeerInfo,
    chain: Blockchain,
    peers: HashSet<PeerInfo>,
    broadcast_channel_in: Sender<(Request, fn(&Response) -> Result<bool>)>,
    broadcast_channel_out: Receiver<(Request, fn(&Response) -> Result<bool>)>,
}

impl Node {
    pub fn new(addr: String) -> Self {
        let (tx, rx) = channel();
        Node {
            basic_info: PeerInfo::new(addr),
            chain: Blockchain::new(),
            peers: HashSet::new(),
            broadcast_channel_in: tx,
            broadcast_channel_out: rx,
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
            "[Node {}] A new block {} is forged, will broadcast it to all peers",
            self.basic_info.id,
            block.get_index()
        );
        // broadcast the newly mined block
        self.async_broadcast_latest_block();
    }

    /// Adds a new transaction
    pub fn create_and_add_new_transaction(&mut self, sender: &str, receiver: &str, amount: i64) {
        let transaction = Transaction::new(sender, receiver, amount);
        if !self.chain.add_new_transaction(&transaction) {
            info!("Transaction already exists");
            return;
        }
        info!(
            "[Node {}] A new transaction is added: {} -> {}, amount: {}",
            self.basic_info.id, sender, receiver, amount
        );
        self.async_broadcast_transaction(transaction);
    }

    // Take an incoming transaction and try to add it
    // If it already exists, drop it and do nothing
    // Else, add and broadcast it
    pub fn handle_incoming_transaction(&mut self, transaction: Transaction) {
        if !self.chain.add_new_transaction(&transaction) {
            debug!("Redundant incoming transaction, simply drop it");
            return;
        }
        self.async_broadcast_transaction(transaction);
    }

    // When a new block comes, check its index:
    // if its index is lower than or equal to that of out latest block, drop it and do nothing
    // if its index is exactly one plus our latest block's index and its previous block is our
    //      latest block, then append it to the end of my chain
    // else, do nothing to this block but then we need to resolve conflicts
    pub fn handle_incoming_block(&mut self, block: Block) {
        if self.chain.add_new_block(&block) {
            // broadcast this good news to my friends~
            self.async_broadcast_latest_block();
        } else {
            // TODO: asynchronously resolve conflicts
        };
    }

    pub fn async_broadcast_transaction(&self, transaction: Transaction) {
        // add this transaction to broadcast channel
        // which will then send it asynchronously
        self.broadcast_channel_in
            .send((
                Request::NewTransaction(self.basic_info.clone(), transaction),
                |resp| {
                    if let Response::Ack(_) = resp {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                },
            ))
            .unwrap();
    }

    pub fn async_broadcast_block(&self, block: Block) {
        self.broadcast_channel_in
            .send((Request::NewBlock(self.get_basic_info(), block), |resp| {
                if let Response::Ack(_) = resp {
                    Ok(true)
                } else {
                    Err(failure::err_msg("Invalid response type"))
                }
            }))
            .unwrap();
    }

    pub fn async_broadcast_latest_block(&self) {
        match self.get_blocks().last() {
            Some(block) => self.async_broadcast_block(block.to_owned()),
            None => debug!("No block to broadcast"),
        }
    }

    pub fn try_fetch_one_broadcast(&self) {
        debug!("try_fetch_one_broadcast...");
        let recv_res = self.broadcast_channel_out.try_recv();
        match recv_res {
            Ok((req, handler)) => {
                debug!("broadcast fetched");
                let _ = self.broadcast_request(&req, handler);
            }
            Err(_) => {}
        }
    }

    // TODO: what does the return value mean?
    fn broadcast_request(
        &self,
        req: &Request,
        response_handler: fn(&Response) -> Result<bool>,
    ) -> Result<bool> {
        let peers = self.peers.clone();
        debug!("broadcasts request {:?} to peers :{:?}", req, peers);
        for peer in peers.iter() {
            debug!("Connecting {:?}", peer);
            let socket_address = peer.get_address().to_socket_addrs().unwrap().as_slice()[0];
            match TcpStream::connect(socket_address) {
                Ok(mut stream) => {
                    serde_json::to_writer(stream.try_clone()?, req)?;
                    stream.flush()?;
                    debug!("Request broadcast, waiting for Response");
                    for response in
                        Deserializer::from_reader(stream.try_clone()?).into_iter::<Response>()
                    {
                        debug!("Response received {:?}", response);
                        let response = response
                            .map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?;
                        let _result = response_handler(&response);
                        break;
                    }
                    debug!("Response handled");
                    // Err(failure::err_msg("No response"))
                }
                Err(e) => {
                    debug!("Connection to {:?} failed: {}", peer, e);
                    // Err(failure::err_msg("Failed to connect"))
                }
            };
            debug!("broadcast to one peer finished");
        }
        // Err(failure::err_msg("No peer to connect"))
        debug!("broadcast finished");
        Ok(true)
    }

    /// Displays the full blockchain
    pub fn display(&self) {
        self.chain.display();
    }

    // TODO: I think this can be merged into `say_hello()`. The name `detect_peer()` is a little bit ambiguous
    /// Adds a new peer. Returns false if `addr` is not a valid socket addr
    pub fn detect_peer(&mut self, addr: &str) -> bool {
        match addr.to_socket_addrs() {
            Ok(addr) => {
                let addr = addr.as_slice();
                assert_eq!(addr.len(), 1);
                match TcpStream::connect(addr[0]) {
                    Ok(stream) => {
                        if let Ok(true) = self.say_hello(stream) {
                            true
                        } else {
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

    pub fn say_hello(&mut self, mut stream: TcpStream) -> Result<bool> {
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

    pub fn add_peer(&mut self, peer: PeerInfo) -> bool {
        if self.peers.contains(&peer) {
            debug!("Peer already exists: {:?}", peer);
            false
        } else {
            debug!("New peer added: {:?}", peer);
            self.peers.insert(peer);
            true
        }
    }

    pub fn update_chain(&mut self, new_blocks: Vec<Block>) -> bool {
        if new_blocks.len() <= self.chain.len() {
            return false;
        }
        let mut new_chain = Blockchain::from_blocks(new_blocks);
        if !Blockchain::valid_chain(&new_chain) {
            return false;
        }
        // add current transactions that are not on the chain yet
        // otherwise, these transaction would be lost!
        for t in self.chain.get_current_transactions() {
            new_chain.add_new_transaction(&t);
        }
        self.chain = new_chain;
        // broadcast only the latest block
        self.async_broadcast_latest_block();
        true
    }

    /// This is our Consensus Algorithm, it resolves conflicts
    /// by replacing our chain with the longest one in the network.
    /// Returns `true` if the chain is replaced
    pub fn resolve_conflicts(&mut self) -> bool {
        let mut ret = false;
        let peers = self.peers.clone();
        debug!("Resolve conflict with peers :{:?}", peers);
        for peer in peers.iter() {
            debug!("Connecting {:?}", peer);
            let socket_address = peer.get_address().to_socket_addrs().unwrap().as_slice()[0];
            match TcpStream::connect(socket_address) {
                Ok(stream) => {
                    debug!("Resolve conflict with peer :{:?}", peer);
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
                Ok(self.update_chain(blocks))
            } else {
                Err(failure::err_msg("Invalid response"))
            };
        }
        return Err(failure::err_msg("No response"));
    }
}
