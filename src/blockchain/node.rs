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

    pub fn broadcast_block(&self, block: Block, from_node: &Node, nodes: &mut [Node]) {
        for peer_id in &self.peers {
            // 1) Procura peer_node
            if let Some(peer_node) = nodes.iter_mut().find(|n| n.node_id == *peer_id) {
                // 2) Chama receive_block passando *somente* block.clone() e from_node
                peer_node.receive_block(block.clone(), from_node);
            }
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
}