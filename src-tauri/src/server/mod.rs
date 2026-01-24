use axum::{
    routing::{get, post},
    Router, Json, response::IntoResponse,
    http::{StatusCode, header},
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::Arc;
use rust_embed::RustEmbed;
use tapi_lib::config::profile::Profile;
use tapi_lib::utils::logger::ProgressLogger;
use tokio::sync::{broadcast, RwLock};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tapi_lib::core::database::{DatabaseManager, HashEntryOutput};

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
    profile: Arc<RwLock<Profile>>,
    db: Arc<RwLock<Option<DatabaseManager>>>,
    _tx: broadcast::Sender<String>,
}

pub async fn start_server(port: u16, host: &str) {
    let (tx, _) = broadcast::channel(100);
    
    // Load profile
    let profile = Arc::new(RwLock::new(Profile::load(Path::new("profile.json")).unwrap_or_default()));
    
    // Initialize DB
    let db_path = PathBuf::from("tapi.db");
    let db = match DatabaseManager::new(db_path).await {
        Ok(dm) => Arc::new(RwLock::new(Some(dm))),
        Err(_) => Arc::new(RwLock::new(None)),
    };
    
    let state = AppState {
        profile,
        db,
        _tx: tx,
    };

    let app = Router::new()
        .route("/api/status", get(status_handler))
        .route("/api/settings/load", get(load_settings))
        .route("/api/settings/save", post(save_settings))
        .route("/api/database/list", get(list_hash_names))
        .route("/api/database/save", post(save_hash_name))
        .route("/api/database/delete", post(delete_hash_entry))
        .route("/api/database/clear", post(clear_all_database))
        .route("/api/database/pull", post(pull_remote_database))
        .route("/api/database/push", post(push_remote_database))
        .route("/api/database/test", post(test_database_connection))
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

async fn status_handler() -> impl IntoResponse {
    "SSE/WebSocket placeholder" 
}

async fn load_settings(axum::extract::State(state): axum::extract::State<AppState>) -> Json<Profile> {
    let profile = state.profile.read().await;
    Json(profile.clone())
}

async fn save_settings(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(new_profile): Json<Profile>
) -> StatusCode {
    let mut p = state.profile.write().await;
    *p = new_profile.clone();
    let _ = p.save(Path::new("profile.json"));
    StatusCode::OK
}

#[derive(Deserialize)]
struct SaveHashRequest {
    hash: String,
    name: String,
    folder: String,
}

async fn save_hash_name(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<SaveHashRequest>
) -> impl IntoResponse {
    let db_lock = state.db.read().await;
    
    if let Some(db) = db_lock.as_ref() {
        match db.save_hash(req.hash, req.name, req.folder).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn list_hash_names(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    let db_lock = state.db.read().await;

    if let Some(db) = db_lock.as_ref() {
        match db.list_all().await {
            Ok(entries) => Json::<Vec<HashEntryOutput>>(entries).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

#[derive(Deserialize)]
struct DeleteHashRequest {
    hash: String,
}

async fn delete_hash_entry(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<DeleteHashRequest>
) -> impl IntoResponse {
    let db_lock = state.db.read().await;

    if let Some(db) = db_lock.as_ref() {
        match db.delete_hash(&req.hash).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn clear_all_database(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    let db_lock = state.db.read().await;

    if let Some(db) = db_lock.as_ref() {
        match db.clear_all().await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn pull_remote_database(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    let (url, token, user, pass) = {
        let p = state.profile.read().await;
        (p.remote_db_url.clone(), p.remote_db_token.clone(), p.remote_db_user.clone(), p.remote_db_pass.clone())
    };

    let db_lock = state.db.read().await;

    if let Some(db) = db_lock.as_ref() {
        match db.pull_from_remote(&url, &token, &user, &pass).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn push_remote_database(
    axum::extract::State(state): axum::extract::State<AppState>
) -> impl IntoResponse {
    let (url, token, user, pass) = {
        let p = state.profile.read().await;
        (p.remote_db_url.clone(), p.remote_db_token.clone(), p.remote_db_user.clone(), p.remote_db_pass.clone())
    };

    let db_lock = state.db.read().await;

    if let Some(db) = db_lock.as_ref() {
        match db.push_to_remote(&url, &token, &user, &pass).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    }
}

async fn test_database_connection(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let (url, token, user, pass) = {
        let p = state.profile.read().await;
        (p.remote_db_url.clone(), p.remote_db_token.clone(), p.remote_db_user.clone(), p.remote_db_pass.clone())
    };

    // Use proper test method - no side effects
    match DatabaseManager::test_remote_connection(&url, &token, &user, &pass).await {
        Ok(_) => "Connection successful!".into_response(),
        Err(e) => (StatusCode::UNAUTHORIZED, e.to_string()).into_response(),
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct CliRequest {
    folder: String,
    model: String,
}

async fn start_cli() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED 
}

