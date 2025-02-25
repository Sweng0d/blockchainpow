use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl Wallet {
    pub fn print_info(&self) {
        println!("Wallet address: {}", self.address);
        println!("Public key: {:?}", self.public_key.serialize());
    }
}

pub fn generate_wallet() -> Wallet {
    let secp = Secp256k1::new();
    let mut rng = OsRng;

    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    let pubkey_serialized = public_key.serialize();

    let mut hasher = Sha256::new();
    hasher.update(pubkey_serialized);
    let result = hasher.finalize();
    let address = hex::encode(result);
    Wallet {
        secret_key,
        public_key,
        address,
    }

}

