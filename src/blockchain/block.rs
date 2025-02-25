use chrono::Utc;
use sha2::{Digest, Sha256};
use hex;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, data: String, previous_hash: String) -> Block {
        let time_created = Utc::now().timestamp();
        let nonce: u64 = 0;

        let hash = calculate_hash(index, time_created, &data, &previous_hash, nonce);

        Block {
            index,
            timestamp: time_created,
            data,
            previous_hash,
            hash,
            nonce,
        }
    }
}

pub fn calculate_hash(index: u64, timestamp: i64, data: &str, previous_hash: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();

    hasher.update(index.to_string());
    hasher.update(timestamp.to_string());
    hasher.update(data);
    hasher.update(previous_hash);
    hasher.update(nonce.to_string());

    let result = hasher.finalize();
    return hex::encode(result)

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_creation() {
        let block = Block::new(0, "Test block!".to_string(), "0".to_string());
  
        assert_eq!(block.index, 0);
        assert_eq!(block.data, "Test block!");
        assert_eq!(block.previous_hash, "0");
        assert_eq!(block.nonce, 0, "nonce should be 0 by default in constructor");
        assert_ne!(block.timestamp, 0, "timestamp should not be zero");
        assert!(!block.hash.is_empty(), "hash must not be empty");
  
        
    }
}