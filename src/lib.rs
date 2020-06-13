//! naïve blockchain

// import external crates
#[macro_use]
extern crate log;
use colored::*;

// color values for pretty console output
const PROMINENT_COLOR: &str = "cyan";
const MSG_COLOR: &str = "yellow";
const ERR_COLOR: &str = "red";
const PROMPT_COLOR: &str = "blue";

// list all modules
mod blockchain;
mod command;
mod message;
mod node;
mod peer;
mod utils;

// bring some inner components out for convenience
use blockchain::{Block, Blockchain, Transaction};
use command::Command;
use message::{Request, Response};
use node::Event;
pub use node::Node; // make it public for main.rs
use peer::PeerInfo;
use utils::*;

pub type Result<T> = std::result::Result<T, failure::Error>;
