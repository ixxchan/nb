mod command;
mod message;
mod node;
mod peer;
mod utils;

// color values for pretty console output
const PROMINENT_COLOR: &str = "cyan";
const MSG_COLOR: &str = "yellow";
const ERR_COLOR: &str = "red";
const PROMPT_COLOR: &str = "blue";

// bring some inner components out for convenience
use crate::*;
use command::Command;
use message::{Request, Response};
use node::Event;
use peer::PeerInfo;
use utils::*;

pub use node::Node;
