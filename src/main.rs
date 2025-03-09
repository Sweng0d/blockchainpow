mod blockchain;
mod wallet;

use std::{net::SocketAddr, sync::{Arc, Mutex}};
use axum::{
    extract::{Json, State},
    routing::{get, post},
    response::IntoResponse,
    Router,
};
use clap::Parser;
use tokio;
use serde_json::{json, to_string_pretty};

use crate::blockchain::block::Block;
use crate::blockchain::node::Node;
use crate::wallet::transaction::Transaction;
// Se quiser broadcast no /transaction ou /mine, use reqwest
// use reqwest::Client;

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

/// GET /chain - exibe chain local
async fn get_chain_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let blocks = &node_guard.blockchain.blocks;

    let chain_obj = json!({
        "length": blocks.len(),
        "blocks": blocks
    });
    let pretty_chain = to_string_pretty(&chain_obj).unwrap();
    (axum::http::StatusCode::OK, [("Content-Type","application/json")], pretty_chain)
}

/// POST /transaction - insere transação no mempool local
async fn receive_transaction_handler(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    node_guard.receive_transaction(tx.clone());
    drop(node_guard);

    // Se quiser broadcastar a transação, descomente e use reqwest
    // let client = Client::new();
    // for peer in &state.peers {
    //     let url = format!("http://{}/transaction", peer);
    //     let _ = client.post(&url).json(&tx).send().await;
    // }

    "Transaction received"
}

/// POST /mine - minera bloco local
async fn mine_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    node_guard.blockchain.add_block();
    let new_block = node_guard.blockchain.blocks.last().unwrap().clone();
    let idx = new_block.index;
    drop(node_guard);

    // Se quiser broadcastar, descomente e use reqwest
    // let client = Client::new();
    // for peer in &state.peers {
    //     let url = format!("http://{}/block", peer);
    //     let _ = client.post(&url).json(&new_block).send().await;
    // }

    format!("Mined new block index={}", idx)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let port = args.port;
    let peers_str = args.peers;
    let peers: Vec<String> = if peers_str.is_empty() {
        vec![]
    } else {
        peers_str.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    let node = Node::new(1);
    let state = AppState {
        node: Arc::new(Mutex::new(node)),
        peers,
    };

    let app = Router::new()
        .route("/chain", get(get_chain_handler))
        .route("/transaction", post(receive_transaction_handler))
        .route("/mine", post(mine_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Nó ouvindo em http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
