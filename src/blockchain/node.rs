use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::block::Block;
use crate::wallet::transaction::{Transaction, TransactionError};
use crate::blockchain::node_registry::{register_id, unregister_id};

use std::sync::{Arc, Mutex};
use std::fmt;

pub type NodeId = u32;

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub blockchain: Blockchain,
    pub peers: Vec<NodeId>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum NodeError {
    DuplicateId(NodeId),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::DuplicateId(id) => write!(f, "ID {} is already in use", id),
        }
    }
}

impl Node {
    /// Construtor que recebe um ID escolhido.
    /// Se o ID estiver em uso, faz panic (poderia retornar Result, se preferir).
    pub fn new(node_id: NodeId) -> Self {
        register_id(node_id).unwrap_or_else(|msg| {
            panic!("Failed to create Node with ID {}: {}", node_id, msg);
        });
        let blockchain = Blockchain::new();
        Self {
            node_id,
            blockchain,
            peers: Vec::new(),
        }
    }

    /// Construtor que gera um ID aleatório até achar um que não seja usado.
    #[allow(dead_code)]
    pub fn new_random_id() -> Self {
        loop {
            let random_id = rand::random::<u32>();
            if register_id(random_id).is_ok() {
                return Self {
                    node_id: random_id,
                    blockchain: Blockchain::new(),
                    peers: Vec::new(),
                };
            }
        }
    }

    /// Pede a chain do outro nó (se quisesse).
    #[allow(dead_code)]
    pub fn request_chain(&self, from: &Node) -> Blockchain {
        from.blockchain.clone()
    }

    /// Envia uma transação para outro nó
    pub fn send_transaction(&mut self, to: &mut Node, tx: Result<Transaction, TransactionError>) {
        match tx {
            Ok(tx_ok) => {
                self.blockchain.add_transaction_to_mempool(tx_ok.clone());
                to.receive_transaction(tx_ok);
            }
            Err(e) => {
                eprintln!("Transaction Rejected: {:?}", e);
            }
        }
    }

    /// Recebe transação, coloca no mempool local
    pub fn receive_transaction(&mut self, tx: Transaction) {
        self.blockchain.add_transaction_to_mempool(tx);
    }

    /// Faz broadcast de um bloco para peers
    /// Agora chamamos peer.receive_block(block.clone()) sem `from_node`.
    pub fn broadcast_block(&mut self, block: Block, all_nodes: &[Arc<Mutex<Node>>]) {
        for &peer_id in &self.peers {
            let mut peer = all_nodes[peer_id as usize].lock().unwrap();
            peer.receive_block(block.clone());
        }
    }

    /// Recebe bloco (sem `from_node`):
    /// - se index == len(), chama add_block_from_network
    /// - se index > len(), avisa que poderíamos pedir chain se quiser
    pub fn receive_block(&mut self, block: Block) {
        let local_len = self.blockchain.blocks.len();
        let remote_index = block.index as usize;

        if remote_index == local_len {
            self.blockchain.add_block_from_network(block);
        } else if remote_index > local_len {
            // Se quiser, poderíamos "request_chain" de quem enviou, mas não temos from_node
            // ou chamamos .replace_chain_if_longer(...) se tivermos a chain do outro.
            println!(
                "Node {} recebeu bloco index={}, mas tem only {} blocos. Precisamos da chain do outro?",
                self.node_id, remote_index, local_len
            );
        }
        // se for menor, ignora (fork mais curto)
    }

    #[allow(dead_code)]
    pub fn add_peer(&mut self, peer_id: NodeId) {
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    #[allow(dead_code)]
    pub fn remove_peer(&mut self, peer_id: NodeId) {
        self.peers.retain(|&id| id != peer_id);
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unregister_id(self.node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::wallet::generate_wallet;
    use crate::wallet::transaction::Transaction;

    /// Teste 1: usa IDs=10 e 11
    #[test]
    fn test_send_transaction_to_mempool() {
        let mut node1 = Node::new(10);
        let mut node2 = Node::new(11);

        node1.peers = vec![11];
        node2.peers = vec![10];

        let wallet1 = generate_wallet();
        let tx1 = Transaction::new_signed(&wallet1, "Bob".to_string(), 30)
            .expect("Failed to create the transaction");

        // node1 -> node2
        node1.send_transaction(&mut node2, Ok(tx1.clone()));

        // node2 deve ter 1 tx
        assert_eq!(node2.blockchain.pending_transactions.len(), 1);
        let received_tx = &node2.blockchain.pending_transactions[0];
        assert_eq!(*received_tx, tx1.clone());

        // node1 também guardou
        assert_eq!(node1.blockchain.pending_transactions[0], tx1.clone());
    }

    /// Teste 2: ID=12
    #[test]
    fn test_mining_locally_includes_transactions() {
        let mut node1 = Node::new(12);

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50)
            .expect("Failed tx1");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200)
            .expect("Failed tx2");

        let tx_invalid_result = Transaction::new_signed(&wallet1, wallet2.address.clone(), 0);
        assert!(tx_invalid_result.is_err());

        node1.receive_transaction(tx1.clone());
        node1.receive_transaction(tx2.clone());

        // minerar
        node1.blockchain.add_block();
        let last_block = node1.blockchain.blocks.last().unwrap();
        assert_eq!(last_block.transactions.len(), 2);
        assert!(last_block.transactions.contains(&tx1));
        assert!(last_block.transactions.contains(&tx2));
    }

    /// Teste 3: IDs=0,1,2
    /// broadcast_block => each peer calls receive_block(block)
    #[test]
    fn test_broadcast_block() {
        use std::sync::{Arc, Mutex};
        let node0 = Arc::new(Mutex::new(Node::new(0)));
        let node1 = Arc::new(Mutex::new(Node::new(1)));
        let node2 = Arc::new(Mutex::new(Node::new(2)));

        // peers
        {
            let mut n0 = node0.lock().unwrap();
            n0.peers = vec![1,2];
        }
        {
            let mut n1 = node1.lock().unwrap();
            n1.peers = vec![0,2];
        }
        {
            let mut n2 = node2.lock().unwrap();
            n2.peers = vec![0,1];
        }

        // transações
        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();
        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50).unwrap();
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200).unwrap();

        // node0 recebe tx e minera
        {
            let mut n0 = node0.lock().unwrap();
            n0.receive_transaction(tx1.clone());
            n0.receive_transaction(tx2.clone());
            n0.blockchain.add_block();
            assert_eq!(n0.blockchain.blocks.len(), 2);
        }
        let last_block = {
            let n0 = node0.lock().unwrap();
            n0.blockchain.blocks.last().unwrap().clone()
        };

        // broadcast
        {
            let mut n0 = node0.lock().unwrap();
            n0.broadcast_block(last_block.clone(), &[node0.clone(), node1.clone(), node2.clone()]);
        }

        // checa se node1 recebeu
        {
            let n1 = node1.lock().unwrap();
            assert_eq!(n1.blockchain.blocks.len(), 2);
            let lb1 = n1.blockchain.blocks.last().unwrap();
            assert_eq!(lb1.transactions.len(), 2);
            assert!(lb1.transactions.contains(&tx1));
            assert!(lb1.transactions.contains(&tx2));
        }
        // checa se node2 recebeu
        {
            let n2 = node2.lock().unwrap();
            assert_eq!(n2.blockchain.blocks.len(), 2);
            let lb2 = n2.blockchain.blocks.last().unwrap();
            assert_eq!(lb2.transactions.len(), 2);
            assert!(lb2.transactions.contains(&tx1));
            assert!(lb2.transactions.contains(&tx2));
        }
    }

    /// Teste 4: replace_longer_chain
    /// Agora sem from_node, precisamos fazer manual
    #[test]
    fn replace_longer_chain() {
        let mut node_short = Node::new(30);
        let mut node_long = Node::new(31);

        assert_eq!(node_short.blockchain.blocks.len(), 1);

        node_long.blockchain.add_block();
        node_long.blockchain.add_block();
        node_long.blockchain.add_block();
        assert_eq!(node_long.blockchain.blocks.len(), 4);

        // pega ultimo bloco do node_long e manda p/ node_short
        let last_block_long = node_long.blockchain.blocks.last().unwrap().clone();
        node_short.receive_block(last_block_long);

        // Agora, node_short vê que "index > short.blocks.len()".
        // No code, imprime msg "Precisamos da chain do outro?".
        // Então no teste, chamamos:
        node_short.blockchain.replace_chain_if_longer(&node_long.blockchain);

        assert_eq!(
            node_short.blockchain.blocks.len(),
            node_long.blockchain.blocks.len(),
            "node_short deve ter substituído a própria chain pela do nodeLong"
        );
        assert_eq!(
            node_short.blockchain.blocks,
            node_long.blockchain.blocks
        );
    }

    #[test]
    fn test_fork_same_index_different_hash() {
        let mut node_a = Node::new(40);
        node_a.blockchain.add_block();
        assert_eq!(node_a.blockchain.blocks.len(), 2);

        let block_normal = node_a.blockchain.blocks.last().unwrap().clone();
        let mut block_fork = block_normal.clone();

        block_fork.transactions.clear();
        block_fork.hash = "fake_hash_of_fork".to_owned();
        // ...
    }

    #[test]
    fn test_blockchain_integrity_is_valid() {
        let mut node = Node::new(50);
        use crate::wallet::wallet::generate_wallet;

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50).unwrap();
        node.receive_transaction(tx1);
        node.blockchain.add_block();

        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 100).unwrap();
        node.receive_transaction(tx2);
        node.blockchain.add_block();

        assert_eq!(node.blockchain.blocks.len(), 3);

        assert!(node.blockchain.is_valid());

        // corrompe
        let mut corrupt_block = node.blockchain.blocks[1].clone();
        corrupt_block.transactions.clear();
        node.blockchain.blocks[1] = corrupt_block;

        assert!(!node.blockchain.is_valid());
    }

    #[test]
    fn test_add_and_remove_peer() {
        let mut node = Node::new(60);
        assert!(node.peers.is_empty());

        node.add_peer(61);
        assert_eq!(node.peers, vec![61]);

        // repete 61 => não duplica
        node.add_peer(61);
        assert_eq!(node.peers, vec![61]);

        node.add_peer(62);
        assert_eq!(node.peers, vec![61,62]);

        node.remove_peer(61);
        assert_eq!(node.peers, vec![62]);

        node.remove_peer(61);
        assert_eq!(node.peers, vec![62]);

        node.remove_peer(62);
        assert!(node.peers.is_empty());
    }
}
