use axum::{
    extract::{State},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use crate::AppState;
use serde_json::json;

pub async fn get_chain_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let blocks = node_guard.blockchain.blocks.clone();
    (StatusCode::OK, Json(blocks))
}

pub async fn mine_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    node_guard.blockchain.add_block();
    let new_block = node_guard.blockchain.blocks.last().unwrap().clone();
    let idx = new_block.index;
    let peers = state.peers.lock().unwrap().clone();
    drop(node_guard);

    // Propagar o bloco em background
    let client = reqwest::Client::new();
    tokio::spawn(async move {
        for peer in peers {
            let url = format!("http://{}/chain/sync", peer);
            let result = client
                .post(&url)
                .json(&new_block)
                .send()
                .await;
            if let Err(e) = result {
                eprintln!("Erro ao propagar bloco para {}: {:?}", peer, e);
            }
        }
    });

    let response = json!({
        "message": "Mined new block",
        "index": idx
    });
    (StatusCode::OK, Json(response))
}

pub async fn get_mempool_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let pending = node_guard.blockchain.pending_transactions.clone();
    let mempool_obj = json!({
        "pending_transactions": pending
    });
    (StatusCode::OK, Json(mempool_obj))
}

pub async fn sync_chain_handler(
    State(state): State<AppState>,
    Json(new_chain): Json<Vec<crate::blockchain::block::Block>>,
) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    let current_length = node_guard.blockchain.blocks.len();
    if new_chain.len() > current_length && node_guard.blockchain.is_valid() {
        println!("Recebida blockchain maior, sincronizando...");
        node_guard.blockchain.blocks = new_chain;
        (StatusCode::OK, Json(json!({"message": "Blockchain synchronized"})))
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({"message": "Invalid or shorter chain"})))
    }
}