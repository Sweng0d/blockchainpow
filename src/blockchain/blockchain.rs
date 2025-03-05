use crate::blockchain::block::Block;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;
use crate::blockchain::block::calculate_hash;
use crate::wallet::wallet::Wallet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: u32,
}

impl Blockchain {
    //create a new blockchain
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty: 4,
        };

        // bloco gênese
        let genesis = Block::new(0, vec![], "0".to_string());
        blockchain.blocks.push(genesis);

        blockchain 
    }

    //add transactions to mempool
    pub fn add_transaction_to_mempool(&mut self, tx: Transaction) {
        if tx.is_valid() {
            self.pending_transactions.push(tx);
        } else {
            println!("Invalid Transaction, ignoring...");
        }
    }

    //add block to the blockchain. It will calculate_hash and mineblock, and afterwards add it to the blockchain.
    pub fn add_block(&mut self) {
        let index = self.blocks.len() as u64;
        
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            &last_block.hash
        } else {
            "0"
        };

        let txs = self.pending_transactions.clone();
        self.pending_transactions.clear();

        let mut new_block = Block::new(index, txs, previous_hash.to_string());
        new_block.mine_block(self.difficulty);

        self.blocks.push(new_block);
    }

    //check if entire blockchain is valid
    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            // Se o previous_hash não bate com o hash do bloco anterior
            if current.previous_hash != previous.hash {
                return false;
            }

            // Recalcular e comparar com o hash guardado
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
            // (Opcional) check PoW se quiser
            // if !current.hash_starts_with_zeros(self.difficulty) { return false; }
        }

        true
    }

    // se existir chain maior e válida, substitui
    pub fn replace_chain_if_longer(&mut self, new_chain: &Blockchain) -> bool {
        if !new_chain.is_valid() {
            return false;
        }

        if new_chain.blocks.len() > self.blocks.len() {
            self.blocks = new_chain.blocks.clone();
            // se quiser, também pegar new_chain.pending_transactions
            true
        } else {
            false
        }
    }

    //cria uma transação assinada e põe no mempool
    pub fn new_signed_tx_and_added_mempool(
        &mut self,
        from_wallet: &Wallet,
        to_address: String,
        amount: u64
    ) {
        let tx = Transaction::new_signed(from_wallet, to_address, amount);
        self.add_transaction_to_mempool(tx);
    }

    //recebe um bloco da rede
    pub fn add_block_from_network(&mut self, block: Block) {
        // se block.index == self.blocks.len(), é o próximo bloco
        if block.index as usize == self.blocks.len() {
            let last_hash = self.blocks.last().unwrap().hash.clone();
            if block.previous_hash == last_hash {
                // (Opcional) Verificar se PoW é válido
                // ex.: if !block.is_valid_pow(self.difficulty) { return; }

                // se tudo ok, adiciona
                self.blocks.push(block);
            }
        } else if block.index as usize > self.blocks.len() {
            // Precisamos da cadeia do outro pra ver se é maior
            println!("A new block has bigger index, we might request the full chain from that node!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation_and_adding_blocks() {
        let mut blockchain = Blockchain::new();
        assert!(blockchain.is_valid()); // deve ser válido mesmo só com gênese
        // ...
    }
}
