use axum::{
    routing::{get, post},
    Router, Json, response::IntoResponse,
    http::{StatusCode, header},
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use rust_embed::RustEmbed;
use tapi_lib::config::profile::Profile;
use tapi_lib::utils::logger::ProgressLogger;
use tokio::sync::broadcast;
use serde::Deserialize;
use std::path::Path;

#[derive(RustEmbed)]
#[folder = "../build/"] // Svelte build output
struct Assets;

#[allow(dead_code)]
struct ServerLogger {
    tx: broadcast::Sender<String>,
}

impl ProgressLogger for ServerLogger {
    fn log(&self, message: String) {
        let _ = self.tx.send(format!("LOG:{}", message));
    }
    fn progress(&self, current: usize, total: usize, message: String) {
        let _ = self.tx.send(format!("PROG:{}:{}:{}", current, total, message));
    }
}

#[derive(Clone)]
struct AppState {
    profile: Arc<Mutex<Profile>>,
    _tx: broadcast::Sender<String>,
}

pub async fn start_server(port: u16, host: &str) {
    let (tx, _) = broadcast::channel(100);
    
    // Load profile
    let profile = Arc::new(Mutex::new(Profile::load(Path::new("profile.json")).unwrap_or_default()));
    
    let state = AppState {
        profile,
        _tx: tx,
    };

    let app = Router::new()
        .route("/api/status", get(status_handler))
        .route("/api/settings/load", get(load_settings))
        .route("/api/settings/save", post(save_settings))
        .route("/api/translate/cli", post(start_cli))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("Invalid address");
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return StatusCode::NOT_FOUND.into_response();
            }
            // SPA fallback - avoid recursion by calling explicitly for index.html content
            match Assets::get("index.html") {
                 Some(content) => {
                     let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                     ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                 },
                 None => StatusCode::NOT_FOUND.into_response()
            }
        }
    }
}

// Handlers implementation needed here...
// Simplified placeholders for now to fit context limit
async fn status_handler() -> impl IntoResponse {
    "SSE/WebSocket placeholder" 
}

async fn load_settings(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Profile> {
    let profile = state.profile.lock().unwrap().clone();
    Json(profile)
}

async fn save_settings(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(new_profile): Json<Profile>
) -> StatusCode {
    if let Ok(mut p) = state.profile.lock() {
        *p = new_profile.clone();
        let _ = p.save(Path::new("profile.json"));
    }
    StatusCode::OK
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct CliRequest {
    folder: String,
    model: String,
    // ... complete args
}

async fn start_cli() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED // Placeholder
}
