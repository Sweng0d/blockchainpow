use secp256k1::{Secp256k1, Message, PublicKey}; 
use secp256k1::ecdsa::Signature; 
use sha2::{Sha256, Digest};
use crate::wallet::wallet::Wallet;
use serde::{Serialize, Deserialize};
use crate::errors::TransactionError; 

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from_address: String,   
    pub to_address: String,
    pub amount: u64,
    pub public_key: Option<PublicKey>,
    pub signature: Option<Signature>,
}

impl Transaction {
    fn payload_string(&self) -> String {
        format!("{}|{}|{}", self.from_address, self.to_address, self.amount)
    }

    //não tem que ser from Wallet to: Wallet?
    pub fn new_signed(from_wallet: &Wallet, to_address: String, amount: u64) -> Result<Transaction, TransactionError> {
        if amount == 0 {
            return Err(TransactionError::InvalidAmount);
        }
        let from_address = from_wallet.address.clone();

        let mut tx = Transaction {
            from_address, 
            to_address,
            amount,
            public_key: Some(from_wallet.public_key),
            signature: None, //Sign later
        };

        let data_string = tx.payload_string();

        let sig = sign_data(from_wallet, data_string.as_bytes());

        tx.signature = Some(sig);

        Ok(tx)
    }

    //to add transactions to the mempool we check if they are valid
    pub fn is_valid(&self) -> bool {
        if self.public_key.is_none() || self.signature.is_none() {
            return false;
        }

        let data_string = self.payload_string();

        let mut hasher = Sha256::new();
        hasher.update(data_string.as_bytes());
        let result = hasher.finalize();

        let message = Message::from_digest_slice(&result).expect("Hash deve ter 32 bytes");

        let secp = Secp256k1::new();
        let sig = self.signature.as_ref().unwrap();
        let pub_key = self.public_key.as_ref().unwrap();

        secp.verify_ecdsa(&message, sig, pub_key).is_ok()
    }

    pub fn tx_hash(&self) -> String {

        let mut data = format!("{}|{}|{}", 
            self.from_address, 
            self.to_address, 
            self.amount
        );

        if let Some(pub_key) = &self.public_key {
            // Converte para bytes comprimidos ou não
            // Normalmente, `pub_key.serialize()` dá 33 bytes (comprimido) ou 65 (descomprimido)
            let pub_key_bytes = pub_key.serialize();
            data.push_str(&hex::encode(pub_key_bytes));
        }

        if let Some(sig) = &self.signature {
            // ECDSA compact: 64 bytes. Some crates usam 72 etc. 
            // Em secp256k1, `sig.serialize_compact()` dá 64 bytes
            let sig_bytes = sig.serialize_compact();
            data.push_str(&hex::encode(sig_bytes));
        }

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }  

}


pub fn sign_data(wallet: &Wallet, data: &[u8]) -> Signature {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let result = hasher.finalize();

    let message = Message::from_digest_slice(&result)
    .expect("Hash deve ter 32 bytes");

    let secp = Secp256k1::new();
    let signature = secp.sign_ecdsa(&message, &wallet.secret_key);

    signature
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::wallet::{generate_wallet}; 
    // Ajuste o import conforme sua estrutura

    #[test]
    fn test_new_signed_valid_transaction() {
        // Cria uma carteira (remetente)
        let from_wallet = generate_wallet();  
        // Cria um endereço de destino (pode ser outra carteira ou string)
        let to_wallet = generate_wallet();  

        // Tenta criar transação com amount=50
        let tx_result = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 50);
        assert!(tx_result.is_ok(), "Esperado que a transação seja criada com sucesso");

        let tx = tx_result.unwrap();

        // Verifica campos básicos
        assert_eq!(tx.from_address, from_wallet.address, "from_address deve ser o do wallet remetente");
        assert_eq!(tx.to_address, to_wallet.address, "to_address deve ser o do wallet destinatário");
        assert_eq!(tx.amount, 50);
        assert!(tx.public_key.is_some(), "A transação deve conter a public_key do remetente");
        assert!(tx.signature.is_some(), "A transação deve estar assinada");

        // Verifica se is_valid() retorna true
        let valid = tx.is_valid();
        assert!(valid, "Transação deve ser considerada válida");
    }

    #[test]
    fn test_new_signed_invalid_amount() {
        let from_wallet = generate_wallet();
        let to_wallet = generate_wallet();

        let tx_result = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 0);
        assert!(tx_result.is_err(), "Esperado erro pois amount=0");

        if let Err(e) = tx_result {
            match e {
                TransactionError::InvalidAmount => {
                    // Erro esperado, teste passa
                }
                TransactionError::InvalidSignature(_) | TransactionError::InvalidTx(_) => {
                    panic!("Esperado TransactionError::InvalidAmount, mas recebeu outro erro");
                }
            }
        }
    }

    #[test]
    fn test_is_valid_missing_signature_or_pubkey() {
        let from_wallet = generate_wallet();
        let to_wallet = generate_wallet();

        // Cria transação válida
        let mut tx = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 10)
            .expect("Deveria criar ok");
        assert!(tx.is_valid(), "Deveria ser válida inicialmente");

        // Remove a assinatura => deve se tornar inválida
        tx.signature = None;
        assert!(!tx.is_valid(), "Sem signature, transação não pode ser válida");

        // Remove também a public_key => continua inválida
        tx.public_key = None;
        assert!(!tx.is_valid(), "Sem public_key, transação não é válida");
    }

    #[test]
    fn test_tx_hash_changes_if_fields_change() {
        let from_wallet = generate_wallet();
        let to_wallet = generate_wallet();

        let tx = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 100)
            .expect("Cria transação com amount=100");
        
        let hash1 = tx.tx_hash();
        assert!(!hash1.is_empty(), "Hash não deve ser vazio");

        // Clone e altera o amount para ver se o hash muda
        let mut tx2 = tx.clone();
        tx2.amount = 200;

        let hash2 = tx2.tx_hash();
        assert_ne!(hash1, hash2, "Hash deve mudar quando amount muda");

        // Retorna o amount para 100
        tx2.amount = 100;
        // Mas se a assinatura depende do payload, nesse caso, a signature do tx2
        // não corresponde mais ao payload. Então is_valid() poderia falhar.
        // Pra fins de hash, só mostrando que o hash depende do amount + signature + pub_key

        // Você também pode alterar a signature e ver se o hash muda
        tx2.signature = None;  // ou outra signature
        let hash3 = tx2.tx_hash();
        assert_ne!(hash1, hash3, "Hash deve mudar se assinatura muda");
    }

    #[test]
    fn test_tx_hash_is_deterministic() {
        let from_wallet = generate_wallet();
        let to_wallet = generate_wallet();

        // Cria duas transações idênticas (mesmo from, to, amount)
        let tx1 = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 10)
            .expect("ok");
        let tx2 = Transaction::new_signed(&from_wallet, to_wallet.address.clone(), 10)
            .expect("ok");

        // Se o from_wallet gera a mesma chave pública e assina do mesmo jeito,
        // as transações devem ter a mesma 'payload' e a mesma signature,
        // mas na prática, a assinatura pode variar levemente se secp256k1 gera Nonces aleatórios
        // Em ECDSA normal, isso pode gerar um hash distinto. 
        // Dependendo do 'nonce' da assinatura, tx_hash pode ser diferente.
        // Para esse teste, assumindo determinismo do sign_data, ou se a sign_data usa RNG estável, 
        // elas podem divergir. 
        
        // Caso a sign_data use RNG, cada transaction terá signature diferente => hash diferente.
        // Então esse teste serve para ilustrar a ideia. Se sua sign_data é determinística, descomente:
        assert_eq!(tx1.tx_hash(), tx2.tx_hash(), "Hash should be same if signature is deterministic");
    }
}
