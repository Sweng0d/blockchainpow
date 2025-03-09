use axum::{
    extract::{Json, State},
    response::IntoResponse,
};

use crate::{AppState};
use crate::wallet::transaction::Transaction;

/// POST /transaction - insere transação no mempool local
pub async fn receive_transaction_handler(
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