mod blockchain;
mod wallet;

use std::{net::SocketAddr, sync::{Arc, Mutex}};
use axum::{
    extract::{State, Json},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clap::Parser;
use tokio;
use serde_json::{json, to_string_pretty};
use reqwest::Client;

// Se quiser broadcast no /mine, habilite no Cargo.toml: reqwest = { version="0.11", features=["json"] }
// use reqwest::Client;

use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::node::Node; // Node => fn receive_block(&mut self, block: Block, from_node: &Node)
use crate::wallet::transaction::Transaction;

#[derive(Debug, Parser)]
#[clap(name="blockchainpow")]
struct Args {
    #[clap(long, default_value="3000")]
    port: u16,

    #[clap(long, default_value="")]
    peers: String,
}

#[derive(Clone)]
struct AppState {
    node: Arc<Mutex<Node>>,
    peers: Vec<String>,
}

/// Ao criar uma cópia do Node, podemos passá-la como `&Node` sem conflitar com o &mut node_guard
/// (evitando o erro E0502).
fn make_node_copy(original: &Node) -> Node {
    Node {
        node_id: original.node_id,
        blockchain: original.blockchain.clone(), // copia o Blockchain
        peers: original.peers.clone(),           // copia lista de peers
    }
}

//------------------------------------------------------------------------------
// Handlers
//------------------------------------------------------------------------------

/// GET /chain: retorna os blocos em JSON "indentado"
async fn get_chain_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let blocks = &node_guard.blockchain.blocks;
    let chain_json = json!({ "length": blocks.len(), "blocks": blocks });
    let pretty_chain = to_string_pretty(&chain_json).unwrap();
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "application/json")],
        pretty_chain
    )
}

/// POST /transaction: insere transação no mempool
async fn receive_transaction_handler(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> impl IntoResponse {
    let mut node = state.node.lock().unwrap();
    // Adiciona a transação ao mempool local
    node.receive_transaction(tx.clone());
    drop(node); // solta o lock antes de fazer as requisições async

    // (Opcional) Agora broadcast para peers
    let client = Client::new();
    for peer in &state.peers {
        let url = format!("http://{}/transaction", peer);
        // Enviamos a mesma transação
        let _ = client.post(&url).json(&tx).send().await;
    }

    "Transaction received and broadcasted"
}

/// POST /block: recebe bloco minerado de outro nó
///
/// `node.rs` define:
///   fn receive_block(&mut self, block: Block, from_node: &Node)
/// Precisamos passar DOIS argumentos:
///   1) &mut self (node_guard)
///   2) &Node (from_node)
///
/// Para não incorrer em E0502 (não podemos usar &*node_guard como from_node),
/// criamos uma CÓPIA do Node (`from_node_copy`), e passamos &from_node_copy.
async fn receive_block_handler(State(state): State<AppState>, Json(block): Json<Block>) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();

    // Cria um Node "cópia" com os campos principais
    let from_node_copy = make_node_copy(&node_guard);

    // Agora chamamos receive_block(block, &from_node_copy)
    //   - `node_guard` é &mut Node
    //   - from_node_copy é um Node separado, então passamos &Node sem conflito
    node_guard.receive_block(block, &from_node_copy);

    "Block received"
}

/// POST /mine: cria bloco local, (opcional) broadcast p/ peers
async fn mine_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    node_guard.blockchain.add_block(); 
    let new_block = node_guard.blockchain.blocks.last().unwrap().clone();
    let idx = new_block.index;
    drop(node_guard);

    // Se quiser broadcastar:
    // let client = Client::new();
    // for peer in &state.peers {
    //     let url = format!("http://{}/block", peer);
    //     let _ = client.post(&url).json(&new_block).send().await;
    // }

    format!("Mined new block index={}", idx)
}

//------------------------------------------------------------------------------
// main
//------------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let port = args.port;
    let peers_str = args.peers;
    let peers: Vec<String> = if peers_str.is_empty() {
        vec![]
    } else {
        peers_str.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Cria Node com ID=1 (ou o que preferir)
    let node = Node::new(1);
    let state = AppState {
        node: Arc::new(Mutex::new(node)),
        peers,
    };

    let app = Router::new()
        .route("/chain", get(get_chain_handler))
        .route("/transaction", post(receive_transaction_handler))
        .route("/block", post(receive_block_handler))
        .route("/mine", post(mine_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0,0,0,0], port));
    println!("Nó ouvindo em http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
