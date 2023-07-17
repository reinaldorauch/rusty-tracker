use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
    routing::get,
    Extension, Form, Router,
};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer, RequireAuthorizationLayer, SqliteStore,
};
use axum_template::{engine::Engine, RenderHtml};
use handlebars::Handlebars;
use serde::Serialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracker::{AuthContext, DisplayUser, ListTorrent, ServerError, User};

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
struct AppState {
    engine: AppEngine,
    app_db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    dotenv::dotenv().ok();

    let secret = std::env::var("SECRET").map_err(ServerError::CouldNotGetSecret)?;

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret.as_bytes()).with_secure(false);

    let mut hbs = Handlebars::new();

    hbs.set_dev_mode(true);
    hbs.register_templates_directory(".hbs", "./templates")
        .expect("Could not register the template dir");

    let db = tracker::connect_db().await?;

    let user_store = SqliteStore::<User>::new(db);
    let auth_layer = AuthLayer::new(user_store, &secret.as_bytes());

    let app_db = tracker::connect_db().await?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a route
    let app = Router::new()
        .route("/torrents", get(torrents_handler))
        .route("/users", get(list_users))
        .route_layer(RequireAuthorizationLayer::<i64, tracker::User>::login())
        .route("/", get(root_handler))
        .route("/login", get(login_handler).post(post_login_handler))
        .route("/logout", get(logout_handler))
        .fallback(not_found)
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            engine: Engine::from(hbs),
            app_db,
        });

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Trying to listen on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| ServerError::CouldNotStartServer(e.to_string()))?;

    Ok(())
}

async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("index", state.engine, ())
}

async fn login_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("login", state.engine, ())
}

async fn logout_handler(mut auth: AuthContext) -> Redirect {
    dbg!("Logging out user:");
    dbg!(&auth.current_user);
    auth.logout().await;
    Redirect::to("/login")
}

async fn post_login_handler(
    auth: AuthContext,
    State(state): State<AppState>,
    Form(input): Form<tracker::LoginData>,
) -> Redirect {
    if tracker::check_login(&state.app_db, auth, input).await {
        Redirect::to("/torrents")
    } else {
        Redirect::to("/login?error=invalid_user")
    }
}

#[derive(Default, Debug, Serialize)]
struct NotFoundData {
    uri: String,
}

async fn not_found(uri: Uri, State(state): State<AppState>) -> (StatusCode, impl IntoResponse) {
    (
        StatusCode::NOT_FOUND,
        RenderHtml(
            "404",
            state.engine,
            NotFoundData {
                uri: uri.to_string(),
            },
        ),
    )
}

#[derive(Default, Debug, Serialize)]
struct ListTorrentsData {
    torrents: Vec<ListTorrent>,
    error: Option<String>,
}

async fn torrents_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let data = match tracker::list_torrents(&state.app_db).await {
        Ok(torrents) => ListTorrentsData {
            torrents,
            error: None,
        },
        Err(err) => ListTorrentsData {
            torrents: vec![],
            error: Some(err.to_string()),
        },
    };
    RenderHtml("torrents", state.engine, data)
}

#[derive(Default, Debug, Serialize)]
struct ListUsersData {
    users: Vec<DisplayUser>,
    error: Option<String>,
}

async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    dbg!("Listing users");
    let data: ListUsersData = match tracker::list_users(&state.app_db).await {
        Ok(users) => ListUsersData { users, error: None },
        Err(err) => ListUsersData {
            users: vec![],
            error: Some(err.to_string()),
        },
    };

    RenderHtml("users", state.engine, data)
}
