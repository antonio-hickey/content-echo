use sqlx::PgPool;
use std::sync::Mutex;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct AppState {
    pub active_cnx: Mutex<u32>,
    pub db_pool: PgPool,
    pub max_payload_size: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenClaims {
    pub id: Uuid,
}
