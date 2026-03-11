use std::net::SocketAddr;
use std::sync::Arc;

use rmm_map::{BBox, LatLon, MapCache, MapData, OverpassClient, Road, GeneratedTrack};
use rmm_server::AppState;

fn test_cache_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rmm-track-api-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn test_state_with_cache(cache_dir: &std::path::Path) -> Arc<AppState> {
    let overpass = OverpassClient::new();
    let cache = MapCache::new(cache_dir);
    Arc::new(AppState::new(overpass, cache))
}

/// Seed cache with a road network that forms a loop (~2km circuit).
fn seed_cache_with_loop(cache: &MapCache, bbox: BBox) {
    // Create a square of roads, each side ~555m
    let data = MapData {
        roads: vec![
            Road {
                id: 1,
                highway_type: "residential".into(),
                points: vec![
                    LatLon::new(51.510, -0.120),
                    LatLon::new(51.510, -0.110),
                ],
                width: 6.0,
                name: Some("South Street".into()),
                oneway: false,
                lanes: 2,
            },
            Road {
                id: 2,
                highway_type: "primary".into(),
                points: vec![
                    LatLon::new(51.510, -0.110),
                    LatLon::new(51.515, -0.110),
                ],
                width: 10.0,
                name: Some("East Road".into()),
                oneway: false,
                lanes: 2,
            },
            Road {
                id: 3,
                highway_type: "tertiary".into(),
                points: vec![
                    LatLon::new(51.515, -0.110),
                    LatLon::new(51.515, -0.120),
                ],
                width: 7.0,
                name: Some("North Lane".into()),
                oneway: false,
                lanes: 2,
            },
            Road {
                id: 4,
                highway_type: "residential".into(),
                points: vec![
                    LatLon::new(51.515, -0.120),
                    LatLon::new(51.510, -0.120),
                ],
                width: 6.0,
                name: Some("West Avenue".into()),
                oneway: false,
                lanes: 2,
            },
        ],
        buildings: vec![],
        water: vec![],
        parks: vec![],
        forests: vec![],
        bbox,
    };
    cache.set(&bbox, &data).unwrap();
}

/// Seed cache with a sparse area (single short road, no loops possible).
fn seed_cache_sparse(cache: &MapCache, bbox: BBox) {
    let data = MapData {
        roads: vec![Road {
            id: 1,
            highway_type: "residential".into(),
            points: vec![
                LatLon::new(51.510, -0.120),
                LatLon::new(51.511, -0.119),
            ],
            width: 6.0,
            name: None,
            oneway: false,
            lanes: 2,
        }],
        buildings: vec![],
        water: vec![],
        parks: vec![],
        forests: vec![],
        bbox,
    };
    cache.set(&bbox, &data).unwrap();
}

async fn spawn_app_with_cache(cache_dir: &std::path::Path) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    let state = test_state_with_cache(cache_dir);
    tokio::spawn(async move {
        axum::serve(listener, rmm_server::app(state)).await.unwrap();
    });

    base_url
}

#[tokio::test]
async fn track_missing_params_returns_400() {
    let cache_dir = test_cache_dir("missing");
    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/track", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn track_returns_generated_track_from_cache() {
    let cache_dir = test_cache_dir("loop");
    let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();

    let cache = MapCache::new(&cache_dir);
    seed_cache_with_loop(&cache, bbox);

    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/api/track?south=51.510&west=-0.120&north=51.515&east=-0.110",
            base
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(content_type, "application/x-msgpack");

    let bytes = resp.bytes().await.unwrap();
    let track: GeneratedTrack = rmp_serde::from_slice(&bytes).unwrap();
    assert!(track.total_length_m > 500.0, "track should have reasonable length");
    assert!(!track.checkpoints.is_empty(), "track should have checkpoints");
    assert!(track.score.total > 0.0, "track should have a positive score");

    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn track_sparse_area_returns_404() {
    let cache_dir = test_cache_dir("sparse");
    let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();

    let cache = MapCache::new(&cache_dir);
    seed_cache_sparse(&cache, bbox);

    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/api/track?south=51.510&west=-0.120&north=51.515&east=-0.110",
            base
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("No raceable track"));
    assert!(body["suggestion"].is_string());

    let _ = std::fs::remove_dir_all(&cache_dir);
}
