mod blockchain;
mod wallet;

// Importe as structs e métodos que precisa
use crate::blockchain::blockchain::Blockchain;
use crate::wallet::wallet::{generate_wallet};
use crate::wallet::transaction::Transaction;

fn main() {
    println!("=== Inicializando Blockchain PoW em Rust ===");

    // 1) Cria o Blockchain (com difficulty = 3, segundo seu código)
    let mut blockchain = Blockchain::new();
    println!("Criado Blockchain com dificuldade = {}", blockchain.difficulty);

    // 2) Gera uma carteira
    let my_wallet = generate_wallet();
    println!("Minha carteira address: {}", my_wallet.address);

    // 3) Criar transações e colocar no mempool
    let tx1 = Transaction::new_signed(&my_wallet, "Alice".to_string(), 10);
    let tx2 = Transaction::new_signed(&my_wallet, "Bob".to_string(), 20);

    // Use a função de inserir no mempool
    blockchain.add_transaction_to_mempool(tx1);
    blockchain.add_transaction_to_mempool(tx2);
    blockchain.new_signed_tx_and_added_mempool(&my_wallet, "Bob".to_string(), 50);

    // 4) Agora “minerar” chamando add_block
    //    Isso pega pending_transactions, cria um bloco, faz PoW e insere no 'blocks'
    println!("Minerando bloco com as transações pendentes...");
    blockchain.add_block();  // Repare que em seu código, add_block() não recebe nada e usa mempool

    // 5) Verificar se a chain continua válida
    println!("Blockchain está válida? {}", blockchain.is_valid());

    // 6) Exibir em JSON
    let json_str = serde_json::to_string_pretty(&blockchain)
        .expect("Falha ao serializar blockchain para JSON");

    println!("=== Blockchain em JSON ===\n{}", json_str);
}
