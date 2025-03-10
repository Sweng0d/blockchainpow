use axum::{
    extract::State,
    response::IntoResponse,
    Json,
    http::StatusCode
};
#[allow(unused_imports)]
use std::collections::HashMap;
use uuid::Uuid;
use serde::Serialize;

use crate::AppState;
use crate::wallet::wallet::generate_wallet; // a função do seu wallet.rs
#[allow(unused_imports)]
use crate::wallet::wallet::Wallet; // se precisar

use serde_json::json;

pub async fn create_wallet_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. Gerar a wallet via generate_wallet()
    let wallet = generate_wallet();

    // 2. Criar um ID único pra identificar essa wallet
    let wallet_id = Uuid::new_v4().to_string();

    // 3. Guardar no HashMap
    {
        let mut guard = state.wallets.lock().unwrap();
        guard.insert(wallet_id.clone(), wallet.clone());
    }

    // 4. Preparar saída JSON
    // Não retornamos secret_key por segurança
    // Convertendo public_key para hex, se quiser
    let public_key_bytes = wallet.public_key.serialize();
    let public_key_hex = hex::encode(public_key_bytes);

    let resp = json!({
        "wallet_id": wallet_id,
        "address": wallet.address,
        "public_key": public_key_hex
    });

    (StatusCode::OK, Json(resp))
}