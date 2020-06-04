use crate::{Block, PeerInfo, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Hello(PeerInfo),
    HowAreYou(PeerInfo),
    NewTransaction(PeerInfo, Transaction),
    NewBlock(PeerInfo, Block),
}

impl Request{
    pub fn get_peer_info(&self) -> &PeerInfo {
        match self {
            Request::Hello(p) | Request::HowAreYou(p) |
            Request::NewTransaction(p, _) | Request::NewBlock(p, _) =>{
                p
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ack(PeerInfo),                  // for Hello, NewTransaction, NewBlock
    MyBlocks(PeerInfo, Vec<Block>), // for HowAreYou
}
