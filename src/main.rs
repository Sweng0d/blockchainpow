mod blockchain;
mod wallet;

use crate::blockchain::node::Node;
use crate::wallet::wallet::generate_wallet;
use crate::wallet::transaction::Transaction;
use serde_json::json;

fn main() {
    println!("=== Simulação de múltiplos nós com Proof of Work ===");

    // Crie um vetor de nós:
    let mut nodes = vec![
        Node::new(1),
        Node::new(2),
        Node::new(3),
    ];

    // Configure peers
    nodes[0].peers = vec![2, 3];
    nodes[1].peers = vec![1, 3];
    nodes[2].peers = vec![1, 2];

    // 2) Gera três carteiras
    let wallet1 = generate_wallet();
    let wallet2 = generate_wallet();
    let wallet3 = generate_wallet();

    println!("Carteira1 address: {}", wallet1.address);
    println!("Carteira2 address: {}", wallet2.address);
    println!("Carteira3 address: {}", wallet3.address);

    // 3) node1 (nodes[0]) cria transação e envia a node2 (nodes[1])
    let tx1 = Transaction::new_signed(&wallet1, "Bob".to_string(), 50)
        .expect("Failed to create the transaction");

    // --- BLOCO para evitar conflito do borrow checker ---
    {
        use std::mem;
        // Tira temporariamente node1 do vetor
        let mut temp_node0 = mem::replace(&mut nodes[0], Node::new(999));

        // Agora chamamos normalmente:
        temp_node0.send_transaction(&mut nodes[1], Ok(tx1));

        // Recoloca node1 no lugar
        nodes[0] = temp_node0;
    }

    // Agora node2 tem transação pendente
    println!("\nNode2 vai minerar bloco com transações pendentes...");
    nodes[1].blockchain.add_block();
    // node2 agora tem 2 blocos (gênese + bloco recém-minerado)

    // 4) node2 broadcasta esse bloco a node1
    let last_block = nodes[1].blockchain.blocks.last().unwrap().clone();
    println!("Node2 broadcasta bloco de index {} para node1", last_block.index);

    // --- BLOCO para evitar conflito no broadcast_block ---
    {
        use std::mem;
        // Tira temporariamente node2 do vetor
        let mut temp_node1 = mem::replace(&mut nodes[1], Node::new(999));

        // Cria um vetor de referências mutáveis para todos os nós
        let mut node_refs = nodes.iter_mut().collect::<Vec<&mut Node>>();

        // Agora chamamos broadcast_block passando o slice de &mut Node
        temp_node1.broadcast_block(last_block, &temp_node1, &mut node_refs[..]);

        // Recoloca node2 no lugar
        nodes[1] = temp_node1;
    }

    // 5) Após o broadcast, renomeamos localmente para imprimir ou analisar
    let node1 = &nodes[0];
    let node2 = &nodes[1];

    println!("\nVerificando se node1 e node2 têm blockchains válidas:");
    println!("Node1 blockchain is valid? {}", node1.blockchain.is_valid());
    println!("Node2 blockchain is valid? {}", node2.blockchain.is_valid());

    // 6) Exibir blockchains em JSON
    let json_node1 = serde_json::to_string_pretty(&node1.blockchain)
        .expect("Fail to serialize node1 chain");
    let json_node2 = serde_json::to_string_pretty(&node2.blockchain)
        .expect("Fail to serialize node2 chain");

    println!("\n=== Node1 Blockchain ===\n{}", json_node1);
    println!("\n=== Node2 Blockchain ===\n{}", json_node2);

    println!("\n=== Fim da simulação ===");
}
