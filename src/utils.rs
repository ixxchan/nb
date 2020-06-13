use crate::*;
use std::net::{SocketAddr, ToSocketAddrs};

pub fn parse_addr(addr: String) -> Result<SocketAddr> {
    Ok(addr.to_socket_addrs().map(|addr| {
        let addr = addr.as_slice();
        assert_eq!(addr.len(), 1);
        addr[0].to_owned()
    })?)
}
