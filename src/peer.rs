use crate::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct PeerInfo {
    id: String,
    address: SocketAddr,
}

impl PeerInfo {
    pub fn new(address: String) -> Result<Self> {
        Ok(PeerInfo {
            id: Uuid::new_v4().to_string(),
            address: parse_addr(address)?,
        })
    }

    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}
