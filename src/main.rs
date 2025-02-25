mod blockchain;
mod wallet;  
use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::wallet::wallet::{Wallet, generate_wallet};
use crate::wallet::transaction::Transaction;

fn main() {
    println!("Initializing Blockchain PoW in Rust...");

    let mut blockchain = Blockchain::new();
    
    blockchain.add_block("Data data".to_string());
    blockchain.add_block("Anything".to_string());
  
    let json_str = serde_json::to_string_pretty(&blockchain)
        .expect("Falha ao serializar");
    
    let mut my_wallet = generate_wallet();
    my_wallet.print_info();

    //println!("Blockchain como JSON:\n{}", json_str);

    

}