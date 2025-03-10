use axum::{
    extract::{State},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use crate::AppState;

pub async fn get_chain_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let blocks = node_guard.blockchain.blocks.clone(); // Clone os dados para evitar referência temporária
    (StatusCode::OK, Json(blocks))
}

pub async fn mine_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mut node_guard = state.node.lock().unwrap();
    node_guard.blockchain.add_block();
    let new_block = node_guard.blockchain.blocks.last().unwrap().clone();
    let idx = new_block.index;
    drop(node_guard);

    let response = serde_json::json!({
        "message": "Mined new block",
        "index": idx
    });
    (StatusCode::OK, Json(response))
}

pub async fn get_mempool_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let pending = node_guard.blockchain.pending_transactions.clone(); // Clone para consistência
    let mempool_obj = serde_json::json!({
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
        (StatusCode::OK, Json(serde_json::json!({"message": "Blockchain synchronized"})))
    } else {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "Invalid or shorter chain"})))
    }
}