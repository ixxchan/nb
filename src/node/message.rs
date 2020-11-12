use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::net::TcpListener;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Hello(PeerInfo),
    HowAreYou(PeerInfo),
    NewTransaction(PeerInfo, Transaction),
    NewBlock(PeerInfo, Block),
    NewPeer(PeerInfo, PeerInfo),
}

impl Request {
    /// Get the `PeerInfo` of the request sender
    pub fn get_sender_peer_info(&self) -> &PeerInfo {
        match self {
            Request::Hello(p)
            | Request::HowAreYou(p)
            | Request::NewTransaction(p, _)
            | Request::NewBlock(p, _)
            | Request::NewPeer(p, _) => p,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ack(PeerInfo),                  // for Hello, NewTransaction, NewBlock
    MyBlocks(PeerInfo, Vec<Block>), // for HowAreYou
}

pub async fn handle_incoming_connections(
    listener: TcpListener,
    sender: Sender<Event>,
) -> Result<()> {
    for stream in listener.incoming() {
        debug!("new incoming connection");
        match stream {
            Ok(stream) => {
                // There should be only one request, but we have to deserialize from a stream in this way
                let mut request = None;
                for _request in
                    Deserializer::from_reader(stream.try_clone()?).into_iter::<Request>()
                {
                    request = Some(
                        _request
                            .map_err(|e| failure::err_msg(format!("Deserializing error {}", e)))?,
                    );
                    debug!("request received {:?}", request);
                    break;
                }
                sender.send(Event::Request(stream, request.unwrap())).await;
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
    Ok(())
}
