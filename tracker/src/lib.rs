use std::env::VarError;

use argon2::PasswordVerifier;
use axum_login::{secrecy::SecretVec, AuthUser, SqliteStore};
use serde::Serialize;
use sqlx::{Pool, Sqlite, SqlitePool};

#[derive(serde::Deserialize, Debug)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct User {
    id: i64,
    password_hash: String,
    username: String,
    full_name: String,
}

impl AuthUser<i64> for User {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
}

#[derive(Debug, Default, Clone, sqlx::FromRow, Serialize)]
pub struct DisplayUser {
    id: i64,
    username: String,
}

pub type AuthContext = axum_login::extractors::AuthContext<i64, User, SqliteStore<User>>;

#[derive(Debug, Default, Clone, sqlx::FromRow, Serialize)]
pub struct ListTorrent {
    id: i64,
    short_name: String,
}

#[derive(Debug)]
pub enum ServerError {
    CouldNotGetDbUrl(VarError),
    CouldNotGetSecret(VarError),
    CouldNotConnectToDb(String),
    CouldNotQueryDb(String),
    CouldNotStartServer(String),
}

impl ToString for ServerError {
    fn to_string(&self) -> String {
        match self {
            Self::CouldNotGetDbUrl(_) => String::from("Could not get the Db Url from env"),
            Self::CouldNotGetSecret(_) => String::from("Could not get secret from env"),
            Self::CouldNotConnectToDb(err) => format!("Could not connect to db: {}", err),
            Self::CouldNotQueryDb(err) => format!("Could not query db: {}", err),
            Self::CouldNotStartServer(err) => format!("Could not start server: {}", err),
        }
    }
}

pub async fn connect_db() -> Result<Pool<Sqlite>, ServerError> {
    let url = std::env::var("DATABASE_URL").map_err(ServerError::CouldNotGetDbUrl)?;
    SqlitePool::connect(&url)
        .await
        .map_err(|e| ServerError::CouldNotConnectToDb(e.to_string()))
}

pub async fn check_login(db: &SqlitePool, mut auth: AuthContext, input: LoginData) -> bool {
    println!("Checking login: {:?}", input);
    match sqlx::query!(
        "SELECT id, username, password_hash, full_name FROM users WHERE username = ?1",
        input.username
    )
    .fetch_one(db)
    .await
    {
        Ok(found_record) => {
            let mut hash = argon2::PasswordHash::new(&found_record.password_hash);

            if let Err(_) = hash {
                return false;
            }

            if !argon2::Argon2::default()
                .verify_password(input.password.as_bytes(), &hash.unwrap())
                .is_ok()
            {
                return false;
            }

            let user = User {
                id: found_record.id,
                password_hash: found_record.password_hash,
                username: found_record.username,
                full_name: found_record.full_name,
            };

            auth.login(&user)
                .await
                .expect("Could not set the user in session");

            true
        }
        Err(login_err) => false,
    }
}

pub async fn list_users(db: &SqlitePool) -> Result<Vec<DisplayUser>, ServerError> {
    sqlx::query!("SELECT id, username FROM users")
        .fetch_all(db)
        .await
        .map(|rows| {
            rows.iter()
                .map(|r| DisplayUser {
                    id: r.id,
                    username: r.username.clone(),
                })
                .collect()
        })
        .map_err(|err| ServerError::CouldNotQueryDb(err.to_string()))
}

pub async fn list_torrents(db: &SqlitePool) -> Result<Vec<ListTorrent>, ServerError> {
    sqlx::query_as!(ListTorrent, "SELECT id, short_name FROM torrents")
        .fetch_all(db)
        .await
        .map_err(|err| ServerError::CouldNotQueryDb(err.to_string()))
}
