use std::net::SocketAddr;
use std::sync::Arc;

use rmm_map::{MapCache, OverpassClient};
use rmm_server::AppState;

fn test_state() -> Arc<AppState> {
    let overpass = OverpassClient::new();
    let cache = MapCache::new(std::env::temp_dir().join("rmm-test-cache"));
    Arc::new(AppState::new(overpass, cache))
}

async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    let state = test_state();
    tokio::spawn(async move {
        axum::serve(listener, rmm_server::app(state)).await.unwrap();
    });

    base_url
}

#[tokio::test]
async fn health_returns_ok() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client.get(format!("{}/health", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ping_returns_ok() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/ping", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}
