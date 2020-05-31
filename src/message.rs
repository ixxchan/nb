use crate::Block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request,
    Response(Vec<Block>),
}
