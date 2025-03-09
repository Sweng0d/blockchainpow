use axum::{
    extract::{State},
    response::IntoResponse,
    http::StatusCode
};
use serde_json::{json, to_string_pretty};

use crate::AppState; // Vamos definir AppState em main.rs, depois importamos aqui

/// GET /chain - exibe chain local
pub async fn get_chain_handler(State(state): State<AppState>) -> impl IntoResponse {
    let node_guard = state.node.lock().unwrap();
    let blocks = &node_guard.blockchain.blocks;

    let chain_obj = json!({
        "length": blocks.len(),
        "blocks": blocks
    });
    let pretty_chain = to_string_pretty(&chain_obj).unwrap();
    (axum::http::StatusCode::OK, [("Content-Type","application/json")], pretty_chain)
}

/// POST /mine - minera bloco local
pub async fn mine_handler(State(state): State<AppState>) -> impl IntoResponse {
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