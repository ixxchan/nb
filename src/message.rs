use crate::{Block, PeerInfo, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Hello(PeerInfo),
    HowAreYou(PeerInfo),
    NewTransaction(PeerInfo, Transaction),
    NewBlock(PeerInfo, Block),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ack(PeerInfo),                  // for Hello, NewTransaction, NewBlock
    MyBlocks(PeerInfo, Vec<Block>), // for HowAreYou
}
