use chrono::Utc;
use sha2::{Digest, Sha256};
use hex;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;
use serde_json;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        let time_created = Utc::now().timestamp();
        let nonce: u64 = 0;

        let hash = calculate_hash(index, time_created, &transactions, &previous_hash, nonce);

        Block {
            index,
            timestamp: time_created,
            transactions,
            previous_hash,
            hash,
            nonce,
        }
    }
}

pub fn calculate_hash(index: u64, timestamp: i64, transactions: &Vec<Transaction>, previous_hash: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();

    hasher.update(index.to_string());
    hasher.update(timestamp.to_string());
    let tx_string = serde_json::to_string(&transactions).unwrap();
    hasher.update(tx_string);
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
        let block = Block::new(0, vec![], "0".to_string());
  
        assert_eq!(block.index, 0);
        assert_eq!(block.transactions.len(), 0);
        assert_eq!(block.previous_hash, "0");
        assert_eq!(block.nonce, 0, "nonce should be 0 by default in constructor");
        assert_ne!(block.timestamp, 0, "timestamp should not be zero");
        assert!(!block.hash.is_empty(), "hash must not be empty");
  
    }
}