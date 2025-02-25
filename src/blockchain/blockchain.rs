use crate::Block;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
        };

        let first_block = Block::new(0, vec![], "0".to_string());
        blockchain.blocks.push(first_block);

        blockchain 
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let index = self.blocks.len() as u64;
        
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            &last_block.hash
        } else {
            "0"
        };

        let new_block = Block::new(
            index,
            transactions,
            previous_hash.to_string(),
        );

        self.blocks.push(new_block);
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation_and_adding_blocks() {

        let mut blockchain = Blockchain::new();

    }
}