use std::net::SocketAddr;
use std::sync::Arc;

use rmm_map::{MapCache, OverpassClient};
use rmm_server::{app, AppState, Config};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rmm_server=info,rmm_map=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env();

    let overpass = OverpassClient::new();
    let cache = MapCache::new(&config.cache_dir);
    if let Err(e) = cache.ensure_dir() {
        tracing::warn!("Failed to create cache directory: {}", e);
    }

    let state = Arc::new(AppState::new(overpass, cache));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app(state)).await.unwrap();
}
