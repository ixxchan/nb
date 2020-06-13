//! naïve blockchain

// import external crates
#[macro_use]
extern crate log;
use colored::*;

// list all modules
mod blockchain;
mod node;

use blockchain::{Block, Blockchain, Transaction};
pub use node::Node; // make it public for main.rs

pub type Result<T> = std::result::Result<T, failure::Error>;
