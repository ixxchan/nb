#[macro_use]
extern crate log;

mod blockchain;
pub mod message;
mod node;

pub use blockchain::{Block, Blockchain};
pub use node::Node;
pub type Result<T> = std::result::Result<T, failure::Error>;
