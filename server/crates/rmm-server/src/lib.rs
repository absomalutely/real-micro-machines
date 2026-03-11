mod config;

pub use config::Config;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn ping() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/ping", get(ping))
        .layer(CorsLayer::permissive())
}
