use axum::{extract::State, response::{Html, IntoResponse, Redirect}, routing::get, Router, Form};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    secrecy::SecretVec, 
    AuthLayer, AuthUser, RequireAuthorizationLayer, SqliteStore,
};
use axum_template::{RenderHtml, engine::Engine};
use handlebars::Handlebars;
use sqlx::SqlitePool;
use tracker::ServerError;
use std::{net::SocketAddr, env::VarError};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
struct AppState {
    engine: AppEngine,
    app_db: SqlitePool
}

#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct User {
    id: u32,
    password_hash: String,
    username: String,
    name: String
}

impl AuthUser<u32> for User {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
}

type AuthContext = axum_login::extractors::AuthContext<i64, User, SqliteStore<User>>;

#[tokio::main]
async fn main() -> Result<(), ServerError>{
    dotenv::dotenv().ok();

    let secret = std::env::var("SECRET").map_err(ServerError::CouldNotGetSecret)?;

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret.as_bytes()).with_secure(false);

    let mut hbs = Handlebars::new();

    hbs.set_dev_mode(true);
    hbs.register_templates_directory(".hbs","./templates")
        .expect("Could not register the template dir");

    let db = tracker::connect_db().await?;

    let user_store = SqliteStore::<User>::new(db);
    let auth_layer = AuthLayer::new(user_store, &secret.as_bytes());

    let app_db = tracker::connect_db().await?;


    // build our application with a route
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/login", get(login_handler).post(post_login_handler))
        .route("/torrents", get(torrents_handler))
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(AppState {
            engine: Engine::from(hbs),
            app_db
        });

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Trying to listen on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await.map_err(|e| ServerError::CouldNotStartServer(e.to_string()))?;

    Ok(())
}

async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("index", state.engine, ())
}

async fn login_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("login", state.engine, ())
}

async fn post_login_handler(State(state): State<AppState>, Form(input): Form<tracker::LoginData>) -> Redirect {
    if tracker::check_login(&state.app_db, input).await {
        Redirect::to("/torrents")
    } else {
        Redirect::to("/login?error=invalid_user")
    }
}

async fn torrents_handler() -> Html<&'static str> {
    Html("<h1>Torrents</h1>")
}