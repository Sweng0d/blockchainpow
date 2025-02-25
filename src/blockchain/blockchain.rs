use crate::Block;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
        };

        let first_block = Block::new(0, "First Block".to_string(), "0".to_string());
        blockchain.blocks.push(first_block);

        blockchain 
    }

    pub fn add_block(&mut self, data: String) {
        let index = self.blocks.len() as u64;
        
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            &last_block.hash
        } else {
            "0"
        };

        let new_block = Block::new(
            index,
            data,
            previous_hash.to_string(),
        );

        self.blocks.push(new_block);
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::Block; // se precisar importá-lo explicitamente

    #[test]
    fn test_blockchain_creation_and_adding_blocks() {

        let mut blockchain = Blockchain::new();

        assert_eq!(blockchain.blocks.len(), 1);

        assert!(blockchain.blocks[0].index == 0); 
        assert!(blockchain.blocks[0].data == "First Block");
        assert!(blockchain.blocks[0].previous_hash == "0");

        blockchain.add_block("second block, blockchain for real.".to_string());
        
        assert_eq!(blockchain.blocks.len(), 2);
        let second_block = &blockchain.blocks[1];
        assert_eq!(second_block.index, 1); 
        assert_eq!(second_block.data, "second block, blockchain for real."); 

    }
}