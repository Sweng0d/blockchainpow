mod blockchain;
mod wallet;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex}
};

use axum::{
    routing::{get, post},
    Router,
    extract::{State, Json},
};
use clap::Parser;
use tokio;

use crate::blockchain::node::Node; // seu Node (contendo blockchain, etc.)
use crate::wallet::transaction::Transaction;

use crate::blockchain::block::Block;
use reqwest::Client; // para fazer POST nos peers
use axum::response::IntoResponse;

#[derive(Debug, Parser)]
#[clap(name = "blockchainpow")]
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

//GET /chain – para consultar a chain local.
//POST /transaction – para enviar transações.
//POST /block – para enviar blocos minerados.

async fn get_chain_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let node = state.node.lock().unwrap();
    let blocks = &node.blockchain.blocks;
    // Retornamos um JSON com "length" e "blocks"
    Json(serde_json::json!({
        "length": blocks.len(),
        "blocks": blocks,
    }))
}

async fn receive_transaction_handler(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> &'static str {
    let mut node = state.node.lock().unwrap();
    node.receive_transaction(tx);
    // se quiser, broadcastar para peers via reqwest
    "Transaction received"
}

async fn receive_block_handler(State(state): State<AppState>) -> &'static str {
    // Precisamos extrair o Block do JSON? Exemplo:
    //    async fn receive_block_handler(State(state): State<AppState>, Json(block): Json<Block>) -> ...
    // Por ora, so print
    let mut node = state.node.lock().unwrap();
    // node.receive_block(block); // se tiver esse método
    "Block received"
}

async fn mine_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut node = state.node.lock().unwrap();

    node.blockchain.add_block(); 
    let new_block = node.blockchain.blocks.last().unwrap().clone();
    let index = new_block.index;
    drop(node);

    // broadcast ...

    // Retorna algo que Axum converte em 200 OK com body
    format!("Mined new block index={}", index)
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

     // Cria Node. Se seu Node::new exigir ID, você pode gerar algo ou parse
     let node = Node::new(1);
     // Embrulha em Arc<Mutex<_>>
     let state = AppState {
         node: Arc::new(Mutex::new(node)),
         peers,
     };
 
     // Definir rotas
     let app = Router::new()
         .route("/chain", get(get_chain_handler))
         .route("/transaction", post(receive_transaction_handler))
         .route("/block", post(receive_block_handler))
         .route("/mine", post(mine_handler))
         .with_state(state);
 
     // Sobe servidor
     let addr = SocketAddr::from(([0,0,0,0], port));
     println!("Nó ouvindo em http://{}", addr);
 
     axum::Server::bind(&addr)
         .serve(app.into_make_service())
         .await
         .unwrap();
 
}
