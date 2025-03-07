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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wallet_fields() {
        let wallet = generate_wallet();
        
        // Verifica se secret_key e public_key não são "vazios"
        // SecretKey e PublicKey não têm um método is_empty, mas podemos conferir se 
        // a serialização tem tamanho esperado.
        let sk_serialized = wallet.secret_key.display_secret().to_string();
        assert!(!sk_serialized.is_empty(), "Secret key não deveria ser vazia");

        let pk_bytes = wallet.public_key.serialize();
        assert!(!pk_bytes.is_empty(), "Public key não deveria ser vazia");

        // Verifica o address
        assert!(!wallet.address.is_empty(), "Endereço não deveria ser vazio");
        
        // Opcional: Conferir tamanho do address hex (64 chars = 32 bytes em hex)
        // Normalmente Sha256 -> 32 bytes => 64 hex
        assert_eq!(wallet.address.len(), 64, "Address deve ter 64 caracteres em hex");
    }

    #[test]
    fn test_generate_wallet_uniqueness() {
        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        // Em princípio, as duas carteiras devem ter chaves e endereços distintos
        // Embora haja uma chance minúscula de colisão, é incrivelmente improvável.
        assert_ne!(wallet1.address, wallet2.address, "Endereços devem ser diferentes na maior parte dos casos");
        assert_ne!(wallet1.secret_key.display_secret().to_string(),
                   wallet2.secret_key.display_secret().to_string(),
                   "Secret keys devem ser diferentes");
    }

    #[test]
    fn test_print_info() {
        let wallet = generate_wallet();
        wallet.print_info();
        // Esse teste apenas chama print_info. 
        // Se quiser capturar a saída, teria que usar um "cargo test -- --nocapture" ou algo similar.
        // Aqui só garantimos que não panic.
    }
}

