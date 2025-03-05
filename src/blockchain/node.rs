use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::block::Block;
use crate::Transaction;

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

    pub fn send_transaction(&self, to: &mut Node, tx: Transaction) {
        to.receive_transaction(tx);
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