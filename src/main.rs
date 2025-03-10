mod blockchain;
mod wallet;
mod routes;
mod errors;

use std::{net::SocketAddr, sync::{Arc, Mutex}};
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use tokio;
use crate::wallet::wallet::Wallet; 
use std::collections::HashMap;

use crate::blockchain::node::Node;
use crate::routes::{
    chain_routes::{get_chain_handler, mine_handler, get_mempool_handler, sync_chain_handler},
    peer_routes::{get_peers_handler, add_peer_handler},
    transaction_routes::{receive_transaction_handler, create_transaction_handler},
    wallet_routes::create_wallet_handler,
};

#[derive(Debug, Parser)]
#[clap(name="blockchainpow")]
struct Args {
    #[clap(long, default_value="3000")]
    port: u16,

    #[clap(long, default_value="")]
    peers: String,
}

#[derive(Clone)]
pub struct AppState {
    pub node: Arc<Mutex<Node>>,
    pub peers: Arc<Mutex<Vec<String>>>,
    pub wallets: Arc<Mutex<HashMap<String, Wallet>>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let port = args.port;

    let peers_str = args.peers;

    let peers_list: Vec<String> = if peers_str.is_empty() {
        vec![]
    } else {
        peers_str.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    let node = Node::new(1);
    let state = AppState {
        node: Arc::new(Mutex::new(node)),
        peers: Arc::new(Mutex::new(peers_list)),
        wallets: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/chain", get(get_chain_handler))
        .route("/mine", post(mine_handler))
        .route("/transaction", post(receive_transaction_handler))
        .route("/transaction/create", post(create_transaction_handler))
        .route("/peers", get(get_peers_handler).post(add_peer_handler))
        .route("/mempool", get(get_mempool_handler))
        .route("/chain/sync", post(sync_chain_handler))
        .route("/wallet/create", post(create_wallet_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Nó ouvindo em http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}