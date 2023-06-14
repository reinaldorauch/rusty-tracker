use axum::{extract::State, response::{Html, IntoResponse, Redirect}, routing::get, Router, Form};
use axum_template::{RenderHtml, engine::Engine};
use handlebars::Handlebars;
use std::net::SocketAddr;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
struct AppState {
    engine: AppEngine,
}

#[tokio::main]
async fn main() {
    let mut hbs = Handlebars::new();

    hbs.set_dev_mode(true);
    hbs.register_templates_directory(".hbs","./templates")
        .expect("Could not register the template dir");

    // build our application with a route
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/login", get(login_handler).post(post_login_handler))
        .route("/torrents", get(torrents_handler))
        .with_state(AppState {
            engine: Engine::from(hbs),
        });

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Trying to listen on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("index", state.engine, ())
}

async fn login_handler(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("login", state.engine, ())
}

#[derive(serde::Deserialize, Debug)]
struct LoginData {
    username: String,
    password: String
}

async fn post_login_handler(Form(input): Form<LoginData>) -> Redirect {
    println!("Got login with data: {:?}", input);
    Redirect::to("/torrents")
}

async fn torrents_handler() -> Html<&'static str> {
    Html("<h1>Torrents</h1>")
}