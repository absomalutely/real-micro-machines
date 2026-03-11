use std::net::SocketAddr;
use std::sync::Arc;

use rmm_map::{BBox, Building, LatLon, MapCache, MapData, OverpassClient, Polygon, Road};
use rmm_server::AppState;

fn test_cache_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rmm-map-api-test-{}",
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

/// Pre-seed cache with fixture data so tests don't hit the live API.
fn seed_cache(cache: &MapCache, bbox: BBox) {
    let data = MapData {
        roads: vec![Road {
            id: 1001,
            highway_type: "residential".into(),
            points: vec![
                LatLon::new(51.511, -0.116),
                LatLon::new(51.512, -0.115),
            ],
            width: 6.0,
            name: Some("Test Street".into()),
            oneway: false,
            lanes: 2,
        }],
        buildings: vec![Building {
            id: 2001,
            footprint: vec![
                LatLon::new(51.511, -0.117),
                LatLon::new(51.511, -0.116),
                LatLon::new(51.512, -0.116),
                LatLon::new(51.511, -0.117),
            ],
            height: 12.0,
        }],
        water: vec![],
        parks: vec![Polygon {
            id: 4001,
            points: vec![
                LatLon::new(51.514, -0.118),
                LatLon::new(51.514, -0.116),
                LatLon::new(51.515, -0.116),
                LatLon::new(51.514, -0.118),
            ],
            polygon_type: "park".into(),
        }],
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
async fn map_missing_params_returns_400() {
    let cache_dir = test_cache_dir();
    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    // No params
    let resp = client
        .get(format!("{}/api/map", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Partial params
    let resp = client
        .get(format!("{}/api/map?south=51.51&west=-0.12", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn map_invalid_bbox_returns_400() {
    let cache_dir = test_cache_dir();
    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    // south > north
    let resp = client
        .get(format!(
            "{}/api/map?south=51.52&west=-0.12&north=51.51&east=-0.11",
            base
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn map_too_small_bbox_returns_400() {
    let cache_dir = test_cache_dir();
    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    // Very tiny bbox (< 100m)
    let resp = client
        .get(format!(
            "{}/api/map?south=51.510&west=-0.1100&north=51.5101&east=-0.1099",
            base
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let _ = std::fs::remove_dir_all(&cache_dir);
}

#[tokio::test]
async fn map_returns_msgpack_from_cache() {
    let cache_dir = test_cache_dir();
    let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();

    // Pre-seed the cache
    let cache = MapCache::new(&cache_dir);
    seed_cache(&cache, bbox);

    let base = spawn_app_with_cache(&cache_dir).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/api/map?south=51.510&west=-0.120&north=51.515&east=-0.110",
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

    // Deserialize the MessagePack response
    let bytes = resp.bytes().await.unwrap();
    let data: MapData = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(data.roads.len(), 1);
    assert_eq!(data.buildings.len(), 1);
    assert_eq!(data.parks.len(), 1);
    assert_eq!(data.roads[0].name.as_deref(), Some("Test Street"));

    let _ = std::fs::remove_dir_all(&cache_dir);
}
