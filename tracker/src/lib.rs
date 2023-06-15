use std::env::VarError;

use axum::headers::Server;
use sqlx::{SqlitePool, Sqlite, Pool};

#[derive(serde::Deserialize, Debug)]
pub struct LoginData {
   pub username: String,
   pub password: String
}

#[derive(Debug)]
pub enum ServerError {
    CouldNotGetDbUrl(VarError),
    CouldNotGetSecret(VarError),
    CouldNotConnectToDb(String),
    CouldNotStartServer(String)
}

pub async fn connect_db() -> Result<Pool<Sqlite>, ServerError>{
    SqlitePool::connect(
        &std::env::var("DATABASE_URL").map_err(ServerError::CouldNotGetDbUrl)?
    ).await.map_err(|e| ServerError::CouldNotConnectToDb(e.to_string()))
}

pub async fn check_login(db: &SqlitePool, input: LoginData) -> bool {
    match sqlx::query!("SELECT password_hash FROM users WHERE username = ?1", input.username).fetch_one(db).await {
        Ok(rec) => true,
        Err(_) => false
    }
}