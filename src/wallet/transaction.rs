use secp256k1::{Secp256k1, Message, PublicKey}; 
use secp256k1::ecdsa::Signature; 
use sha2::{Sha256, Digest};
use crate::wallet::wallet::Wallet;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from_address: String,   
    pub to_address: String,
    pub amount: u64,
    #[serde(skip)]
    pub public_key: Option<PublicKey>,
    #[serde(skip)]
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone)]
pub enum TransactionError {
    InvalidAmount,
    // Can have other errors -> InvalidSignature etc
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

        let message = Message::from_slice(&result).expect("Hash deve ter 32 bytes");

        let secp = Secp256k1::new();
        let sig = self.signature.as_ref().unwrap();
        let pub_key = self.public_key.as_ref().unwrap();

        secp.verify_ecdsa(&message, sig, pub_key).is_ok()
    }
}


pub fn sign_data(wallet: &Wallet, data: &[u8]) -> Signature {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let result = hasher.finalize();

    let message = Message::from_slice(&result).expect("Hash deve ter 32 bytes");

    let secp = Secp256k1::new();
    let signature = secp.sign_ecdsa(&message, &wallet.secret_key);

    signature
}

