// src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockId { 
    pub height: u64, 
    pub hash: String 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PbftMsg {
    Proposal { height: u64, proposer: u16, block: serde_json::Value },
    Prepare  { from: u16, bid: BlockId },
    Commit   { from: u16, bid: BlockId },
}

// ✅ Basic acknowledgment structs go here
#[derive(Debug, Serialize)]
pub struct Ack { 
    pub ok: bool, 
    pub node_id: u16 
}

#[derive(Debug, Serialize)]
pub struct BroadcastReport { 
    pub sender_id: u16, 
    pub attempted: usize, 
    pub succeeded: usize 
}