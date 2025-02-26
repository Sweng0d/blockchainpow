mod blockchain;
mod wallet;  
use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::wallet::wallet::{Wallet, generate_wallet};
use crate::wallet::transaction::Transaction;

fn main() {
    println!("Initializing Blockchain PoW in Rust...");

    let mut blockchain = Blockchain::new();
    println!("Created new blockchain with difficulty = {}", blockchain.difficulty);

    let mut my_wallet = generate_wallet();
    println!("My wallet address: {}", my_wallet.address);

    let tx1 = Transaction::new_signed(&my_wallet, "Bob".to_string(), 10);
    let tx2 = Transaction::new_signed(&my_wallet, "Alice".to_string(), 20);
    println!("Mining block with 2 transactions...");
    blockchain.add_block(vec![tx1, tx2]);

    println!("Is blockchain valid? {}", blockchain.is_valid());
  
    let json_str = serde_json::to_string_pretty(&blockchain)
        .expect("Failed to serialize blockchain to JSON");

    println!("Blockchain as JSON:\n{}", json_str);

    

}