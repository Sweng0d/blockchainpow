mod blockchain;
mod wallet;

use crate::blockchain::node::Node;
use crate::wallet::wallet::generate_wallet;
use crate::wallet::transaction::Transaction;

use std::sync::{Arc, Mutex};

fn main() {
    println!("=== Simulação de múltiplos nós com Proof of Work ===");

    // 1) Crie três nós protegidos por Arc<Mutex<>> e coloque-os num Vec
    let nodes: Vec<Arc<Mutex<Node>>> = vec![
        Arc::new(Mutex::new(Node::new(1))),
        Arc::new(Mutex::new(Node::new(2))),
        Arc::new(Mutex::new(Node::new(3))),
    ];

    // 2) Configure peers
    //    Cada nó tem peers = IDs dos outros (por exemplo, node1 -> 2,3 etc.)
    {
        let mut n0 = nodes[0].lock().unwrap();
        n0.peers = vec![1, 2]; // ou [2, 3] se preferir
    }
    {
        let mut n1 = nodes[1].lock().unwrap();
        n1.peers = vec![0, 2]; // ou [1, 3]
    }
    {
        let mut n2 = nodes[2].lock().unwrap();
        n2.peers = vec![0, 1]; // ou [1, 2]
    }

    // 3) Gera três carteiras
    let wallet1 = generate_wallet();
    let wallet2 = generate_wallet();
    let wallet3 = generate_wallet();

    println!("Carteira1 address: {}", wallet1.address);
    println!("Carteira2 address: {}", wallet2.address);
    println!("Carteira3 address: {}", wallet3.address);

    // 4) node1 cria transação e envia a node2
    let tx1 = Transaction::new_signed(&wallet1, "Bob".to_string(), 50)
        .expect("Failed to create the transaction");

    let tx_hash = tx1.tx_hash();
    println!("tx1 hash is {}", tx_hash);

    {
        // Trave node1 e node2 para usar &mut Node
        let mut node1_lock = nodes[0].lock().unwrap();
        let mut node2_lock = nodes[1].lock().unwrap();

        // Chame o método send_transaction normalmente
        // (Assumindo que send_transaction é algo como fn send_transaction(&mut self, to: &mut Node, ...))
        node1_lock.send_transaction(&mut *node2_lock, Ok(tx1));
    }

    // 5) node2 minera bloco
    {
        let mut node2_lock = nodes[1].lock().unwrap();
        println!("\nNode2 vai minerar bloco com transações pendentes...");
        node2_lock.blockchain.add_block();
        // Agora node2 tem 2 blocos (gênese + bloco recém-minerado)
    }

    // 6) Pegue o último bloco minerado para broadcast
    let last_block = {
        let node2_lock = nodes[1].lock().unwrap();
        node2_lock.blockchain.blocks.last().unwrap().clone()
    };
    println!(
        "Node2 broadcasta bloco de index {} para node1",
        last_block.index
    );

    // 7) Broadcast do bloco
    {
        let mut node2_lock = nodes[1].lock().unwrap();

        // Precisamos passar a lista de nós para broadcast_block;
        // Então a função broadcast_block deve aceitar algo como
        // broadcast_block(&mut self, block: Block, all_nodes: &[Arc<Mutex<Node>>])
        node2_lock.broadcast_block(last_block, &nodes);
    }

    // 8) Verificar a validade da blockchain do node1 e node2
    {
        let node1_lock = nodes[0].lock().unwrap();
        println!(
            "\nNode1 blockchain is valid? {}",
            node1_lock.blockchain.is_valid()
        );
    }
    {
        let node2_lock = nodes[1].lock().unwrap();
        println!(
            "Node2 blockchain is valid? {}",
            node2_lock.blockchain.is_valid()
        );
    }

    // 9) Exibir blockchains em JSON
    {
        let node1_lock = nodes[0].lock().unwrap();
        let json_node1 = serde_json::to_string_pretty(&node1_lock.blockchain)
            .expect("Fail to serialize node1 chain");
        println!("\n=== Node1 Blockchain ===\n{}", json_node1);
    }
    {
        let node2_lock = nodes[1].lock().unwrap();
        let json_node2 = serde_json::to_string_pretty(&node2_lock.blockchain)
            .expect("Fail to serialize node2 chain");
        println!("\n=== Node2 Blockchain ===\n{}", json_node2);
    }

    println!("\n=== Fim da simulação ===");
}
