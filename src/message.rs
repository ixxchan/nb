use serde::{Deserialize, Serialize};
use crate::Block;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request,
    Response(Vec<Block>),
}