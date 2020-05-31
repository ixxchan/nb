#[macro_use]
extern crate log;

mod blockchain;
mod node;
pub mod message;

pub use blockchain::{Block,Blockchain};
pub use node::Node;
pub type Result<T> = std::result::Result<T, failure::Error>;