use axum::{
    extract::{Json, State},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize};
use serde_json::{json, to_string_pretty};

use crate::AppState;

/// GET /peers - lista os peers conhecidos
pub async fn get_peers_handler(State(state): State<AppState>) -> impl IntoResponse {
    // 1. Tranque o Mutex
    let guard = state.peers.lock().unwrap();
    
    // 2. Agora use &*guard, que é um &Vec<String>, no json!()
    let peers_obj = json!({ 
        "peers": &*guard 
    });
    
    let pretty_peers = to_string_pretty(&peers_obj).unwrap();
    (
        StatusCode::OK,
        [("Content-Type","application/json")],
        pretty_peers
    )
}

#[derive(Deserialize)]
pub struct PeerPayload {
    pub peer: String,
}

/// POST /peers - adiciona um novo peer
pub async fn add_peer_handler(
    State(state): State<AppState>,
    Json(payload): Json<PeerPayload>,
) -> impl IntoResponse {
    // Aqui, se for Arc<Mutex<Vec<String>>>, precisamos travar antes de adicionar
    let mut peers_guard = state.peers.lock().unwrap();
    peers_guard.push(payload.peer);
    drop(peers_guard);

    (StatusCode::OK, "Peer adicionado com sucesso")
}