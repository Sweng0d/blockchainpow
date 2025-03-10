use std::fmt;
use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub enum TransactionError {
    InvalidAmount,
    InvalidSignature(String),
    InvalidTx(String),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::InvalidAmount => write!(f, "Invalid transaction amount"),
            TransactionError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            TransactionError::InvalidTx(msg) => write!(f, "Invalid transaction: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {}

impl IntoResponse for TransactionError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            TransactionError::InvalidAmount => (StatusCode::BAD_REQUEST, "Invalid amount".to_string()),
            TransactionError::InvalidSignature(msg) => (StatusCode::BAD_REQUEST, msg),
            TransactionError::InvalidTx(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, error_message).into_response()
    }
}