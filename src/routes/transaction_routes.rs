use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use crate::AppState;
use crate::wallet::transaction::Transaction;
use crate::errors::TransactionError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    wallet_id: String,
    to_address: String,
    amount: u64,
}

pub async fn create_transaction_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<impl IntoResponse, TransactionError> {
    let wallets = state.wallets.lock().unwrap();
    let wallet = wallets.get(&request.wallet_id)
        .ok_or(TransactionError::InvalidTx("Wallet not found".to_string()))?;
    
    let tx = Transaction::new_signed(wallet, request.to_address, request.amount)?;
    
    let mut node = state.node.lock().unwrap();
    node.verify_signature(&tx)?;
    node.receive_transaction(tx.clone());
    
    Ok(Json(serde_json::json!({
        "message": "Transaction created and added to mempool",
        "transaction": tx
    })))
}

pub async fn receive_transaction_handler(
    State(state): State<AppState>,
    Json(tx): Json<Transaction>,
) -> Result<impl IntoResponse, TransactionError> {
    let mut node = state.node.lock().unwrap();
    node.verify_signature(&tx)?;
    node.receive_transaction(tx);
    Ok("Transaction received")
}