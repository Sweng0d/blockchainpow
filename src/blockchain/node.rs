use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::block::Block;
use crate::Transaction;
use crate::generate_wallet;
use crate::wallet::transaction::TransactionError;

pub type NodeId = u32; 

#[derive(Debug, Clone)]
pub struct Node {
    pub node_id: NodeId,
    pub blockchain: Blockchain,
    pub peers: Vec<NodeId>,
    // pub inbox: Vec<NetworkMessage>, // se quiser simular mensagens
}

impl Node {
    pub fn new(node_id: NodeId) -> Self {
        let blockchain = Blockchain::new(); // se Blockchain::new() já cria bloco gênese
        Self {
            node_id,
            blockchain,
            peers: Vec::new(),
        }
    }

    pub fn request_chain(&self, from: &Node) -> Blockchain {
        from.blockchain.clone()
    }

    pub fn send_transaction(&mut self, to: &mut Node, tx: Result<Transaction, TransactionError>) {
        match tx {
            Ok(tx) => {
                self.blockchain.add_transaction_to_mempool(tx.clone());
                to.receive_transaction(tx);
            }
            Err(e) => {
                eprintln!("Transaction Rejected: {:?}", e);
            }
        }        
    }

    pub fn receive_transaction(&mut self, tx: Transaction) {
        // Coloca no mempool do "to" node
        self.blockchain.add_transaction_to_mempool(tx);
    }

    pub fn broadcast_block(&self, block: Block, from_node: &Node, nodes: &mut [&mut Node]) {
        for peer_node in nodes.iter_mut() {
            peer_node.receive_block(block.clone(), from_node);
        }
    }
    
    pub fn receive_block(&mut self, block: Block, from_node: &Node) {
        // Se block.index == self.blockchain.blocks.len(), adicionamos
        if block.index as usize == self.blockchain.blocks.len() {
            self.blockchain.add_block_from_network(block);
        } 
        // Se o bloco tiver índice maior
        else if block.index as usize > self.blockchain.blocks.len() {
            // Pedimos a chain do from_node
            let remote_chain = from_node.blockchain.clone();
            self.blockchain.replace_chain_if_longer(&remote_chain);
        }
    }

}

#[derive(Debug)]
pub enum NetworkMessage {
    TransactionMessage(Transaction),
    // futuramente: BlockMessage(Block), etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_transaction_to_mempool() {
        let mut node1 = Node::new(1);
        let mut node2 = Node::new(2);

        node1.peers = vec![2];
        node2.peers = vec![1];

        let wallet1 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, "Bob".to_string(), 30).expect("Failed to create the transaction");
        node1.send_transaction(&mut node2, Ok(tx1.clone()));

        assert_eq!(node2.blockchain.pending_transactions.len(), 1);

        let received_tx = &node2.blockchain.pending_transactions[0];
        assert_eq!(*received_tx, tx1.clone());

        assert_eq!(node1.blockchain.pending_transactions[0], tx1.clone());
    }

    #[test]
    fn test_mining_locally_includes_transactions() {
        let mut node1 = Node::new(1);

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50).expect("Failed to create the transaction");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200).expect("Failed to create the transaction");
        let tx_invalid_result = Transaction::new_signed(&wallet1, wallet2.address.clone(), 0);

        assert!(
            tx_invalid_result.is_err(),
            "Transação com amount=0 deveria falhar e retornar Err"
        );

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

    #[test]
    fn test_broadcast_block() {
        let mut node1 = Node::new(1);
        let mut node2 = Node::new(2);
        let mut node3 = Node::new(3);

        node1.peers = vec![2, 3];
        node2.peers = vec![1, 3];
        node3.peers = vec![1, 2];

        let wallet1 = generate_wallet();
        let wallet2 = generate_wallet();

        let tx1 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 50).expect("Failed to create the transaction");
        let tx2 = Transaction::new_signed(&wallet1, wallet2.address.clone(), 200).expect("Failed to create the transaction");

        node1.receive_transaction(tx1.clone());
        node1.receive_transaction(tx2.clone());

        node1.blockchain.add_block();
        assert_eq!(node1.blockchain.blocks.len(), 2, "Node1 has genesis + 1 block");

        let last_block = node1
        .blockchain
        .blocks
        .last()
        .expect("Node1 deve ter pelo menos 2 blocos")
        .clone();

        {
            use std::mem;
            // técnica para “pegar emprestado” node1 sem conflitar
            let mut temp_node1 = mem::replace(&mut node1, Node::new(999));
            temp_node1.broadcast_block(last_block.clone(), &temp_node1, &mut [&mut node2, &mut node3]);
            // Recoloca node1
            node1 = temp_node1;
        }

        assert_eq!(node2.blockchain.blocks.len(), 2, "Node2 received the block via broadcast");
        let last_block_node2 = node2.blockchain.blocks.last().unwrap();
        assert_eq!(last_block_node2.transactions.len(), 2);
        assert!(last_block_node2.transactions.contains(&tx1));
        assert!(last_block_node2.transactions.contains(&tx2));

        assert_eq!(
            node3.blockchain.blocks.len(),
            2,
            "Node3 received the block via broadcast"
        );
        let last_block_node3 = node3.blockchain.blocks.last().unwrap();
        assert_eq!(last_block_node3.transactions.len(), 2);
        assert!(last_block_node3.transactions.contains(&tx1));
        assert!(last_block_node3.transactions.contains(&tx2));
    }

    #[test]
    fn replace_longer_chain() {
        let mut nodeShort = Node::new(1);
        let mut nodeLong = Node::new(2);

        assert_eq!(nodeShort.blockchain.blocks.len(), 1);

        nodeLong.blockchain.add_block();
        nodeLong.blockchain.add_block();
        nodeLong.blockchain.add_block();

        assert_eq!(nodeLong.blockchain.blocks.len(), 4);

        assert_eq!(nodeShort.blockchain.blocks.len(), 1);

        let last_block_node_long = nodeLong.blockchain.blocks.last().unwrap().clone();
        nodeShort.receive_block(last_block_node_long, &nodeLong);

        // 5) O código de receive_block verifica se
        //    block.index > self.blockchain.blocks.len()
        //    e chama replace_chain_if_longer se for o caso.
        //    Com 4 blocos contra 1, deve substituir.

        // 6) Verifique se agora nodeShort tem 4 blocos
        assert_eq!(
        nodeShort.blockchain.blocks.len(),
        nodeLong.blockchain.blocks.len(),
        "nodeShort deve ter substituído a própria chain pela do nodeLong"
        );

        let short_last_block = nodeShort.blockchain.blocks.last().unwrap();
        let long_last_block = nodeLong.blockchain.blocks.last().unwrap();
        assert_eq!(
        short_last_block.hash,
        long_last_block.hash,
        "O último bloco de ambos deve ser igual após o replace"
        );

        assert_eq!(nodeShort.blockchain.blocks, nodeLong.blockchain.blocks);
    }

    #[test]
    fn test_fork_same_index_different_hash() {

        let mut node_a = Node::new(1);

        node_a.blockchain.add_block();
        assert_eq!(node_a.blockchain.blocks.len(), 2);

        let block_normal = node_a.blockchain.blocks.last().cloned().expect("There is a block");

        let mut block_fork = block_normal.clone();

        block_fork.transactions = vec![];

        block_fork.hash = "fake_hash_of_fork".to_string();
    }

    #[test]
fn test_blockchain_integrity_is_valid() {
    let mut node = Node::new(1);

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

    assert_eq!(node.blockchain.blocks.len(), 3, "Expected 3 blocks (Genesis + 2)");

    assert!(
        node.blockchain.is_valid(),
        "A blockchain deve ser válida após blocos minerados corretamente"
    );

    let mut corrupt_block = node.blockchain.blocks[1].clone();
    corrupt_block.transactions.clear();

    node.blockchain.blocks[1] = corrupt_block;

    assert!(
        !node.blockchain.is_valid(),
        "Alterar dados de um bloco deve invalidar toda a chain"
    );
}

}