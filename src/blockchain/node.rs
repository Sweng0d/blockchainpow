use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::block::Block;
use crate::wallet::transaction::Transaction;
use crate::errors::TransactionError; // Ajustado para usar o módulo errors
use crate::blockchain::node_registry::{register_id, unregister_id};
use std::sync::{Arc, Mutex};
use std::fmt;
use secp256k1::{Secp256k1, Message};
use sha2::{Sha256, Digest};

/// Ajuste aqui se quiser mudar o tipo do ID.
pub type NodeId = u32;

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub blockchain: Blockchain,
    pub peers: Vec<NodeId>,
}

/// Possíveis erros ao criar nós
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

    /// Pede a chain do outro nó
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
    pub fn broadcast_block(&mut self, block: Block, all_nodes: &[Arc<Mutex<Node>>]) {
        for &peer_id in &self.peers {
            let mut peer = all_nodes[peer_id as usize].lock().unwrap();
            peer.receive_block(block.clone(), self);
        }
    }

    /// Recebe bloco: se o índice bater com len() local, adiciona;
    /// se for maior, tenta replace_chain_if_longer
    pub fn receive_block(&mut self, block: Block, from_node: &Node) {
        let local_len = self.blockchain.blocks.len();
        let remote_index = block.index as usize;

        if remote_index == local_len {
            self.blockchain.add_block_from_network(block);
        } else if remote_index > local_len {
            let remote_chain = from_node.blockchain.clone();
            self.blockchain.replace_chain_if_longer(&remote_chain);
        }
        // se for menor, ignora (é um fork mais curto)
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

    pub fn verify_signature(&self, tx: &Transaction) -> Result<(), TransactionError> {
        let public_key = tx
            .public_key
            .as_ref()
            .ok_or(TransactionError::InvalidTx("Missing public key".to_string()))?;
        let signature = tx
            .signature
            .as_ref()
            .ok_or(TransactionError::InvalidTx("Missing signature".to_string()))?;

        // Transformar (from_address, to_address, amount) em um hash
        let data_string = format!("{}|{}|{}", tx.from_address, tx.to_address, tx.amount);
        let mut hasher = Sha256::new();
        hasher.update(data_string.as_bytes());
        let result = hasher.finalize();

        // Criar mensagem a partir do hash
        let message = Message::from_digest_slice(&result).expect("Hash deve ter 32 bytes");

        // Verificar a assinatura usando secp256k1
        let secp = Secp256k1::new();
        secp.verify_ecdsa(&message, signature, public_key)
            .map_err(|_| TransactionError::InvalidSignature("Signature does not match".to_string()))?;

        Ok(())
    }
}

/// Quando o `Node` sai de escopo, liberamos o ID no registro
impl Drop for Node {
    fn drop(&mut self) {
        unregister_id(self.node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::wallet::generate_wallet;

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

        node1.send_transaction(&mut node2, Ok(tx1.clone()));
        assert_eq!(node2.blockchain.pending_transactions.len(), 1);
        let received_tx = &node2.blockchain.pending_transactions[0];
        assert_eq!(*received_tx, tx1);
        assert_eq!(node1.blockchain.pending_transactions[0], tx1);
    }

    /// Teste 2: usa ID=12
    #[test]
    fn test_mining_locally_includes_transactions() {
        let mut node1 = Node::new(12);

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50)
            .expect("Failed to create the transaction");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200)
            .expect("Failed to create the transaction");
        let tx_invalid_result = Transaction::new_signed(&wallet1, wallet2.address.clone(), 0);

        assert!(tx_invalid_result.is_err(), "Transação com amount=0 deveria falhar");
        node1.receive_transaction(tx1.clone());
        node1.receive_transaction(tx2.clone());

        node1.blockchain.add_block();
        let last_block = node1
            .blockchain
            .blocks
            .last()
            .expect("Haverá pelo menos o bloco gênese e o bloco minerado");

        assert_eq!(last_block.transactions.len(), 2);
        assert!(last_block.transactions.contains(&tx1));
        assert!(last_block.transactions.contains(&tx2));
    }

    /// Teste 3: usa IDs=20,21,22
    #[test]
    fn test_broadcast_block() {
        let node0 = Arc::new(Mutex::new(Node::new(0)));
        let node1 = Arc::new(Mutex::new(Node::new(1)));
        let node2 = Arc::new(Mutex::new(Node::new(2)));

        {
            let mut n0 = node0.lock().unwrap();
            n0.peers = vec![1, 2];
        }
        {
            let mut n1 = node1.lock().unwrap();
            n1.peers = vec![0, 2];
        }
        {
            let mut n2 = node2.lock().unwrap();
            n2.peers = vec![0, 1];
        }

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50)
            .expect("Failed to create tx1");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200)
            .expect("Failed to create tx2");

        let last_block = {
            let mut n0 = node0.lock().unwrap();
            n0.receive_transaction(tx1.clone());
            n0.receive_transaction(tx2.clone());
            n0.blockchain.add_block();
            assert_eq!(n0.blockchain.blocks.len(), 2);
            n0.blockchain.blocks.last().unwrap().clone()
        };

        {
            let mut n0 = node0.lock().unwrap();
            n0.broadcast_block(last_block.clone(), &[node0.clone(), node1.clone(), node2.clone()]);
        }

        {
            let n1 = node1.lock().unwrap();
            assert_eq!(n1.blockchain.blocks.len(), 2);
            let last_block_node1 = n1.blockchain.blocks.last().unwrap();
            assert_eq!(last_block_node1.transactions.len(), 2);
            assert!(last_block_node1.transactions.contains(&tx1));
            assert!(last_block_node1.transactions.contains(&tx2));
        }

        {
            let n2 = node2.lock().unwrap();
            assert_eq!(n2.blockchain.blocks.len(), 2);
            let last_block_node2 = n2.blockchain.blocks.last().unwrap();
            assert_eq!(last_block_node2.transactions.len(), 2);
            assert!(last_block_node2.transactions.contains(&tx1));
            assert!(last_block_node2.transactions.contains(&tx2));
        }
    }

    /// Teste 4: usa IDs=30 e 31
    #[test]
    fn replace_longer_chain() {
        let mut node_short = Node::new(30);
        let mut node_long = Node::new(31);

        assert_eq!(node_short.blockchain.blocks.len(), 1);

        node_long.blockchain.add_block();
        node_long.blockchain.add_block();
        node_long.blockchain.add_block();

        assert_eq!(node_long.blockchain.blocks.len(), 4);
        assert_eq!(node_short.blockchain.blocks.len(), 1);

        let last_block_node_long = node_long.blockchain.blocks.last().unwrap().clone();
        node_short.receive_block(last_block_node_long, &node_long);

        assert_eq!(
            node_short.blockchain.blocks.len(),
            node_long.blockchain.blocks.len(),
            "node_short deve ter substituído a própria chain pela do nodeLong"
        );

        let short_last_block = node_short.blockchain.blocks.last().unwrap();
        let long_last_block = node_long.blockchain.blocks.last().unwrap();
        assert_eq!(
            short_last_block.hash,
            long_last_block.hash,
            "O último bloco de ambos deve ser igual após o replace"
        );

        assert_eq!(node_short.blockchain.blocks, node_long.blockchain.blocks);
    }

    /// Teste 5: ID=40
    #[test]
    fn test_fork_same_index_different_hash() {
        let mut node_a = Node::new(40);

        node_a.blockchain.add_block();
        assert_eq!(node_a.blockchain.blocks.len(), 2);

        let block_normal = node_a
            .blockchain
            .blocks
            .last()
            .cloned()
            .expect("Deveria haver um bloco");

        let mut block_fork = block_normal.clone();
        block_fork.transactions = vec![];
        block_fork.hash = "fake_hash_of_fork".to_string();
        // Teste incompleto no original, mantido como está
    }

    /// Teste 6: ID=50
    #[test]
    fn test_blockchain_integrity_is_valid() {
        let mut node = Node::new(50);

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50)
            .expect("Failed to create tx1");
        node.receive_transaction(tx1);
        node.blockchain.add_block();

        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 100)
            .expect("Failed to create tx2");
        node.receive_transaction(tx2);
        node.blockchain.add_block();

        assert_eq!(node.blockchain.blocks.len(), 3);
        assert!(node.blockchain.is_valid());

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
        node.add_peer(61);
        assert_eq!(node.peers, vec![61]);
        node.add_peer(62);
        assert_eq!(node.peers, vec![61, 62]);
        node.remove_peer(61);
        assert_eq!(node.peers, vec![62]);
        node.remove_peer(61);
        assert_eq!(node.peers, vec![62]);
        node.remove_peer(62);
        assert!(node.peers.is_empty());
    }
}