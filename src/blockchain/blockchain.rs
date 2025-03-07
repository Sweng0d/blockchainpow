use crate::blockchain::block::Block;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;
use crate::blockchain::block::calculate_hash;
use crate::wallet::wallet::Wallet;
use std::collections::HashMap;
use crate::generate_wallet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>, //mempool
    pub difficulty: u32,
    #[serde(skip)]
    pub tx_map: HashMap<String, Transaction>,
}

impl Blockchain {
    //create a new blockchain
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty: 3,
            tx_map: HashMap::new(),
        };

        // genesis block
        let genesis = Block::new(0, vec![], "0".to_string());
        blockchain.blocks.push(genesis);

        blockchain 
    }

    //add transactions to mempool
    pub fn add_transaction_to_mempool(&mut self, tx: Transaction) {
        let txid = tx.tx_hash();

        if tx.is_valid() {
            self.tx_map.insert(txid, tx.clone());
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
        if let Ok(tx_ok) = tx {
            self.add_transaction_to_mempool(tx_ok);
        } else {
            println!("Failed to create signed transaction");
        }
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

    pub fn find_transaction(&self, tx_hash: &str) -> Option<&Transaction> {
        self.tx_map.get(tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_add_valid_transaction_to_mempool_and_find() {
        let mut blockchain = Blockchain::new();

        // Cria 2 carteiras de teste (ajuste se necessário)
        let wallet_from = generate_wallet(); 
        let wallet_to   = generate_wallet();

        // Cria uma transação válida (amount>0)
        let tx = Transaction::new_signed(&wallet_from, wallet_to.address.clone(), 50)
            .expect("Transação deve ser válida com amount=50");
        
        // Pega o hash da transação
        let tx_hash = tx.tx_hash();

        // Adiciona ao mempool
        blockchain.add_transaction_to_mempool(tx);

        // Verifica se está no tx_map
        let found = blockchain.find_transaction(&tx_hash);
        assert!(found.is_some(), "A transação deve estar no tx_map");
        let found_tx = found.unwrap();
        assert_eq!(found_tx.tx_hash(), tx_hash, "Os hashes devem bater");
        
        // Verifica se está na pending_transactions (mempool)
        assert_eq!(blockchain.pending_transactions.len(), 1, "Deveria haver 1 tx pendente");
        let mempool_tx = &blockchain.pending_transactions[0];
        assert_eq!(mempool_tx.tx_hash(), tx_hash, "Hash no mempool deve bater");
    }

    #[test]
    fn test_add_invalid_transaction_does_not_index() {
        let mut blockchain = Blockchain::new();
        let wallet_from = generate_wallet();
        let wallet_to   = generate_wallet();

        // Cria transação com amount=0 => deve falhar
        let tx_result = Transaction::new_signed(&wallet_from, wallet_to.address.clone(), 0);
        assert!(tx_result.is_err(), "Transação com amount=0 deve retornar Err");

        // Se o tx_result é Err, a gente não chega a inserir nada. 
        // Mas vamos criar manualmente uma tx inválida sem is_valid().
        let invalid_tx = Transaction {
            from_address: wallet_from.address.clone(),
            to_address: wallet_to.address.clone(),
            amount: 0,
            public_key: Some(wallet_from.public_key),
            signature: None, // sem assinar
        };

        // Adiciona ao mempool
        blockchain.add_transaction_to_mempool(invalid_tx);

        // Verifica se mempool continua vazio 
        // (pois transaction.is_valid() deve retornar false)
        assert_eq!(blockchain.pending_transactions.len(), 0, 
            "Nenhuma transação deve ter sido adicionada ao mempool");

        // Verifica se tx_map está vazio também
        assert_eq!(blockchain.tx_map.len(), 0, 
            "Nenhuma transação deve ter sido indexada no tx_map");
    }

    #[test]
    fn test_new_signed_tx_and_added_mempool() {
        let mut blockchain = Blockchain::new();
        let wallet_from = generate_wallet();
        let wallet_to   = generate_wallet();

        // Chama a função do Blockchain que cria e adiciona a tx
        blockchain.new_signed_tx_and_added_mempool(&wallet_from, wallet_to.address.clone(), 25);

        // Se foi válida, deve estar no mempool
        assert_eq!(blockchain.pending_transactions.len(), 1);
        
        // E também no tx_map
        assert_eq!(blockchain.tx_map.len(), 1);

        // Pega a transação do mempool
        let mempool_tx = &blockchain.pending_transactions[0];
        let txid = mempool_tx.tx_hash();

        // Tenta encontrar no tx_map
        let found = blockchain.find_transaction(&txid);
        assert!(found.is_some());
        let found_tx = found.unwrap();

        // Verifica se confere o valor
        assert_eq!(found_tx.amount, 25);
        assert_eq!(found_tx.from_address, wallet_from.address);
        assert_eq!(found_tx.to_address, wallet_to.address);
    }
}
