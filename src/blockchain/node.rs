use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::block::Block;
use crate::wallet::transaction::{Transaction, TransactionError};
use crate::blockchain::node_registry::{register_id, unregister_id};

use std::sync::{Arc, Mutex};
use std::fmt;

/// Ajuste aqui se quiser mudar o tipo do ID.
pub type NodeId = u32;

#[derive(Debug)]
pub struct Node {
    pub node_id: NodeId,
    pub blockchain: Blockchain,
    pub peers: Vec<NodeId>,
}

/// Possíveis erros ao criar nós
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
        // Verifica e registra a ID (fazendo unwrap para simplificar).
        // Se já estiver em uso, panic.
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
    pub fn new_random_id() -> Self {
        loop {
            let random_id = rand::random::<u32>();
            if register_id(random_id).is_ok() {
                // Se conseguiu registrar, sai do loop
                return Self {
                    node_id: random_id,
                    blockchain: Blockchain::new(),
                    peers: Vec::new(),
                };
            }
            // caso contrário, repete
        }
    }

    /// request the chain from other node
    pub fn request_chain(&self, from: &Node) -> Blockchain {
        from.blockchain.clone()
    }

    /// send the transaction to another node
    pub fn send_transaction(&mut self, to: &mut Node, tx: Result<Transaction, TransactionError>) {
        match tx {
            Ok(tx_ok) => {
                // add the tx to the local mempool
                self.blockchain.add_transaction_to_mempool(tx_ok.clone());
                // the other node receive the tx
                to.receive_transaction(tx_ok);
            }
            Err(e) => {
                eprintln!("Transaction Rejected: {:?}", e);
            }
        }
    }

    /// receive the tx and add it to the mempool
    pub fn receive_transaction(&mut self, tx: Transaction) {
        self.blockchain.add_transaction_to_mempool(tx);
    }

    // broadcast the block to other peers
    pub fn broadcast_block(&mut self, block: Block, all_nodes: &[Arc<Mutex<Node>>]) {
        for &peer_id in &self.peers {
            let mut peer = all_nodes[peer_id as usize].lock().unwrap();
            peer.receive_block(block.clone(), self);
        }
    }

    // receive the block from other node and decides whether it adds or replace
    pub fn receive_block(&mut self, block: Block, from_node: &Node) {
        let local_len = self.blockchain.blocks.len();
        let remote_index = block.index as usize;

        if remote_index == local_len {
            self.blockchain.add_block_from_network(block);
        } else if remote_index > local_len {
            let remote_chain = from_node.blockchain.clone();
            self.blockchain.replace_chain_if_longer(&remote_chain);
        }
        // se for menor, ignora
    }

    pub fn add_peer(&mut self, peer_id: NodeId) {
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    pub fn remove_peer(&mut self, peer_id: NodeId) {
        self.peers.retain(|&id| id != peer_id);
    }
}

/// Quando o `Node` sai de escopo, liberamos o ID no registro
impl Drop for Node {
    fn drop(&mut self) {
        unregister_id(self.node_id);
    }
}

// NetworkMessage se precisar
#[derive(Debug)]
pub enum NetworkMessage {
    TransactionMessage(Transaction),
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::transaction::Transaction;
    use crate::wallet::wallet::generate_wallet;
    use std::sync::{Arc, Mutex};

    // Teste 1: usa ID=10 e ID=11
    #[test]
    fn test_send_transaction_to_mempool() {
        let mut node1 = Node::new(10);
        let mut node2 = Node::new(11);

        node1.peers = vec![11];
        node2.peers = vec![10];

        let wallet1 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, "Bob".to_string(), 30)
            .expect("Failed to create the transaction");

        // Envia transação node1 -> node2
        node1.send_transaction(&mut node2, Ok(tx1.clone()));

        // Verifica se node2 recebeu
        assert_eq!(node2.blockchain.pending_transactions.len(), 1);

        let received_tx = &node2.blockchain.pending_transactions[0];
        assert_eq!(*received_tx, tx1.clone());

        // Verifica se node1 também guardou em seu mempool
        assert_eq!(node1.blockchain.pending_transactions[0], tx1.clone());
    }

    // Teste 2: usa ID=12
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

        assert!(
            tx_invalid_result.is_err(),
            "Transação com amount=0 deveria falhar e retornar Err"
        );

        node1.receive_transaction(tx1.clone());
        node1.receive_transaction(tx2.clone());

        // Minerar
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

    // Teste 3: usa IDs=20,21,22
        #[test]
    fn test_broadcast_block() {
        use std::sync::{Arc, Mutex};
        use crate::wallet::transaction::Transaction; // se ainda não estiver importado
        use crate::wallet::wallet::generate_wallet;   // se ainda não estiver importado

        // Cria nó0, nó1, nó2 com IDs 0, 1 e 2
        let node0 = Arc::new(Mutex::new(Node::new(0)));
        let node1 = Arc::new(Mutex::new(Node::new(1)));
        let node2 = Arc::new(Mutex::new(Node::new(2)));

        // Ajusta peers coerentemente:
        // node0 -> peers = [1,2]
        {
            let mut n0 = node0.lock().unwrap();
            n0.peers = vec![1, 2];
        }
        // node1 -> peers = [0,2]
        {
            let mut n1 = node1.lock().unwrap();
            n1.peers = vec![0, 2];
        }
        // node2 -> peers = [0,1]
        {
            let mut n2 = node2.lock().unwrap();
            n2.peers = vec![0, 1];
        }

        // Gera carteiras e transações
        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50)
            .expect("Failed to create tx1");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200)
            .expect("Failed to create tx2");

        // node0 recebe essas transações e minera um bloco
        {
            let mut n0 = node0.lock().unwrap();
            n0.receive_transaction(tx1.clone());
            n0.receive_transaction(tx2.clone());

            n0.blockchain.add_block();
            assert_eq!(
                n0.blockchain.blocks.len(), 
                2,
                "Node0 deveria ter bloco gênese + 1 bloco minerado"
            );
        }

        // Pega o último bloco minerado em node0
        let last_block = {
            let n0 = node0.lock().unwrap();
            n0.blockchain.blocks.last().unwrap().clone()
        };

        // node0 faz broadcast do bloco para [node0, node1, node2]
        {
            let mut n0 = node0.lock().unwrap();
            n0.broadcast_block(
                last_block.clone(),
                &[node0.clone(), node1.clone(), node2.clone()]
            );
        }

        // Verifica se node1 recebeu o bloco
        {
            let n1 = node1.lock().unwrap();
            assert_eq!(
                n1.blockchain.blocks.len(), 
                2, 
                "Node1 deve ter recebido o bloco via broadcast"
            );
            let last_block_node1 = n1.blockchain.blocks.last().unwrap();
            assert_eq!(last_block_node1.transactions.len(), 2);
            assert!(last_block_node1.transactions.contains(&tx1));
            assert!(last_block_node1.transactions.contains(&tx2));
        }

        // Verifica se node2 recebeu o bloco
        {
            let n2 = node2.lock().unwrap();
            assert_eq!(
                n2.blockchain.blocks.len(),
                2,
                "Node2 deve ter recebido o bloco via broadcast"
            );
            let last_block_node2 = n2.blockchain.blocks.last().unwrap();
            assert_eq!(last_block_node2.transactions.len(), 2);
            assert!(last_block_node2.transactions.contains(&tx1));
            assert!(last_block_node2.transactions.contains(&tx2));
        }
    }


    // Teste 4: IDs=30 e 31
    #[test]
    fn replace_longer_chain() {
        let mut node_short = Node::new(30);
        let mut node_long = Node::new(31);

        assert_eq!(node_short.blockchain.blocks.len(), 1);

        // nodeLong faz vários blocos
        node_long.blockchain.add_block();
        node_long.blockchain.add_block();
        node_long.blockchain.add_block();

        assert_eq!(node_long.blockchain.blocks.len(), 4);
        assert_eq!(node_short.blockchain.blocks.len(), 1);

        // Passa o último bloco de nodeLong para nodeShort
        let last_block_node_long = node_long.blockchain.blocks.last().unwrap().clone();
        node_short.receive_block(last_block_node_long, &node_long);

        // Se o índice do bloco for maior do que node_short já tem,
        // node_short deve substituir a chain inteira
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

    // Teste 5: ID=40
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

        // Simulação de "fork": mexe em transações e no hash
        block_fork.transactions = vec![];
        block_fork.hash = "fake_hash_of_fork".to_string();

        // (Você poderia testar se node_a.receive_block(block_fork, ...) faz algo especial.)
    }

    // Teste 6: ID=50
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

        assert_eq!(
            node.blockchain.blocks.len(),
            3,
            "Esperado 3 blocos (Genesis + 2)"
        );

        assert!(
            node.blockchain.is_valid(),
            "A blockchain deve ser válida após blocos minerados corretamente"
        );

        // Corrompe o bloco 1
        let mut corrupt_block = node.blockchain.blocks[1].clone();
        corrupt_block.transactions.clear();

        node.blockchain.blocks[1] = corrupt_block;

        // Deve invalidar a chain
        assert!(
            !node.blockchain.is_valid(),
            "Alterar dados de um bloco deve invalidar toda a chain"
        );
    }
    
    fn test_add_and_remove_peer() {
        // Cria um Node com ID 0 (pode ser qualquer ID que você queira)
        let mut node = Node::new(0);
    
        // No início, peers deve estar vazio
        assert!(node.peers.is_empty());
    
        // Adiciona peer 1
        node.add_peer(1);
        assert_eq!(node.peers, vec![1], "Depois de add_peer(1), peers deve ter [1]");
    
        // Tenta adicionar peer 1 de novo (não deve duplicar)
        node.add_peer(1);
        assert_eq!(node.peers, vec![1], "Não deve ter duplicado o peer 1");
    
        // Adiciona peer 2
        node.add_peer(2);
        assert_eq!(node.peers, vec![1, 2], "Agora deve ter [1, 2]");
    
        // Remove peer 1
        node.remove_peer(1);
        assert_eq!(node.peers, vec![2], "Depois de remover peer 1, deve restar [2]");
    
        // Tenta remover peer 1 novamente, mas ele já não existe
        node.remove_peer(1);
        assert_eq!(node.peers, vec![2], "Remover peer que não existe não muda nada");
    
        // Remove peer 2
        node.remove_peer(2);
        assert!(node.peers.is_empty(), "Lista de peers deve ficar vazia novamente");
    }
}
