use axum::{
    extract::State,
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use uuid::Uuid;
use crate::AppState;
use crate::wallet::wallet::generate_wallet;
use serde_json::json;

pub async fn create_wallet_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let wallet = generate_wallet();
    let wallet_id = Uuid::new_v4().to_string();
    {
        let mut guard = state.wallets.lock().unwrap();
        guard.insert(wallet_id.clone(), wallet.clone());
    }
    let public_key_bytes = wallet.public_key.serialize();
    let public_key_hex = hex::encode(public_key_bytes);
    let resp = json!({
        "wallet_id": wallet_id,
        "address": wallet.address,
        "public_key": public_key_hex
    });
    (StatusCode::OK, Json(resp))
}