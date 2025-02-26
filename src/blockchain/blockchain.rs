use crate::Block;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;
use crate::blockchain::block::calculate_hash;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: u32,
}

impl Blockchain {
    pub fn new() -> Self {
        
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            difficulty: 3,
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

        let mut new_block = Block::new(
            index,
            transactions,
            previous_hash.to_string(),
        );
        new_block.mine_block(self.difficulty);

        self.blocks.push(new_block);
        
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }
    
            let recalculated = calculate_hash(
                current.index,
                current.timestamp,
                &current.transactions,
                &current.previous_hash,
                current.nonce
            );
    
            if recalculated != current.hash {
                return false;
            }   
        }

        true
        
    }

    pub fn replace_chain_if_longer(&mut self, new_chain: &Blockchain) -> bool {
        if !new_chain.is_valid() {
            return false;
        }

        if new_chain.blocks.len() > self.blocks.len() {
            self.blocks = new_chain.blocks.clone();
            true
        } else {
            false
        }
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