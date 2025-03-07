use chrono::Utc;
use sha2::{Digest, Sha256};
use hex;
use serde::{Serialize, Deserialize};
use crate::wallet::transaction::Transaction;
use serde_json;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    //create a new block
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

    //use that to add_block to the blockchain
    pub fn mine_block(&mut self, difficulty: u32) {
        let target_prefix = "0".repeat(difficulty as usize);

        loop {
            self.hash = calculate_hash(
                self.index,
                self.timestamp,
                &self.transactions,
                &self.previous_hash,
                self.nonce
            );

            if self.hash.starts_with(&target_prefix) {
                break;
            }
            self.nonce += 1;
        }
    }

    //verifica o hash do bloco e se a dificuldade está de acordo
    pub fn is_valid(&self, difficulty: u32) -> bool {
        // 1) Recalcular o hash com base em index, timestamp, transactions, previous_hash, nonce
        let recalculated = calculate_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.previous_hash,
            self.nonce
        );
        if recalculated != self.hash {
            return false;
        }

        // 2) Checar se o hash começa com zeros (PoW)
        let target_prefix = "0".repeat(difficulty as usize);
        self.hash.starts_with(&target_prefix)
    }
}

//calculate hash, used in mine_block above
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
    use crate::wallet::transaction::Transaction; // se precisar
    // Se precisar criar transações de teste, importe também a wallet ou algo do tipo

    #[test]
    fn test_block_creation() {
        let block = Block::new(0, vec![], "0".to_string());

        assert_eq!(block.index, 0, "Index inicial deve ser 0");
        assert_eq!(block.transactions.len(), 0, "Sem transações no construtor");
        assert_eq!(block.previous_hash, "0");
        assert_eq!(block.nonce, 0, "nonce deve ser 0 por default");
        assert_ne!(block.timestamp, 0, "timestamp não deve ser zero");
        assert!(!block.hash.is_empty(), "hash deve ser preenchido");
    }

    #[test]
    fn test_mine_block_increases_nonce_and_validates() {
        let mut block = Block::new(1, vec![], "hash-do-bloco-anterior".to_string());
        let difficulty = 2;

        // O hash inicial não necessariamente começa com "00"
        assert!(!block.hash.starts_with("00"));

        // Realiza a mineração
        block.mine_block(difficulty);

        // Verifica se agora começa com "00"
        let prefix = "0".repeat(difficulty as usize);
        assert!(block.hash.starts_with(&prefix), 
            "Hash deve começar com {} zeros após minerar", difficulty);

        // O nonce deve ter aumentado (não ser zero)
        assert!(block.nonce > 0, "nonce deve ser incrementado durante a mineração");

        // is_valid(difficulty) deve retornar true
        assert!(block.is_valid(difficulty), "Bloco deve ser válido após minerar com essa dificuldade");
    }

    #[test]
    fn test_is_valid_after_tampering_nonce() {
        let mut block = Block::new(2, vec![], "prev-hash".to_string());
        let difficulty = 1;
        block.mine_block(difficulty);

        // Confirma que bloco está válido
        assert!(block.is_valid(difficulty));

        // Tenta adulterar o nonce
        block.nonce += 1;

        // Agora deve falhar na verificação
        assert!(!block.is_valid(difficulty), 
            "Ao modificar nonce, o bloco deve ficar inválido");
    }

    #[test]
    fn test_is_valid_after_tampering_transactions() {
        // Vamos criar um bloco com 2 transações de teste
        let tx1 = Transaction {
            from_address: "Alice".into(),
            to_address: "Bob".into(),
            amount: 50,
            public_key: None,
            signature: None,
        };
        let tx2 = Transaction {
            from_address: "Carol".into(),
            to_address: "Dave".into(),
            amount: 100,
            public_key: None,
            signature: None,
        };
        let mut block = Block::new(3, vec![tx1.clone(), tx2.clone()], "prev-hash".to_string());

        let difficulty = 1;
        block.mine_block(difficulty);

        // Bloco deve ser válido
        assert!(block.is_valid(difficulty));

        // Agora adulteramos a primeira transação (ex.: amount = 999)
        block.transactions[0].amount = 999;

        // Com essa adulteração, o hash guardado em 'block.hash' não corresponde mais. 
        // is_valid deve retornar falso
        assert!(!block.is_valid(difficulty), 
            "Modificar as transações deve invalidar o bloco");
    }

    #[test]
    fn test_is_valid_after_tampering_previous_hash() {
        let mut block = Block::new(4, vec![], "prev-hash-abc".to_string());
        let difficulty = 1;
        block.mine_block(difficulty);

        assert!(block.is_valid(difficulty));

        // Altera o previous_hash
        block.previous_hash = "tampered-hash".to_string();
        assert!(!block.is_valid(difficulty), 
            "Modificar o previous_hash deve invalidar o bloco");
    }
}
