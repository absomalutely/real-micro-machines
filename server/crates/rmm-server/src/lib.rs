mod config;

pub use config::Config;

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rmm_map::{BBox, MapCache, MapError, OverpassClient};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tracing::{error, info};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub overpass: OverpassClient,
    pub cache: MapCache,
}

impl AppState {
    pub fn new(overpass: OverpassClient, cache: MapCache) -> Self {
        Self { overpass, cache }
    }
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn ping() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Debug, Deserialize)]
struct MapQuery {
    south: Option<f64>,
    west: Option<f64>,
    north: Option<f64>,
    east: Option<f64>,
}

async fn get_map(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MapQuery>,
) -> impl IntoResponse {
    // Validate all params are present
    let (south, west, north, east) = match (params.south, params.west, params.north, params.east) {
        (Some(s), Some(w), Some(n), Some(e)) => (s, w, n, e),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing required query parameters: south, west, north, east" })),
            )
                .into_response();
        }
    };

    // Create and validate bbox
    let bbox = match BBox::new(south, west, north, east) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Validate size (100m to 2000m)
    if let Err(e) = bbox.validate_size(100.0, 2000.0) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    // Check cache
    if let Some(cached_data) = state.cache.get(&bbox) {
        info!("Serving cached map data for bbox {}", bbox.to_overpass_string());
        return match rmp_serde::to_vec(&cached_data) {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/x-msgpack")],
                bytes,
            )
                .into_response(),
            Err(e) => {
                error!("Failed to serialize cached data: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal server error" })),
                )
                    .into_response()
            }
        };
    }

    // Fetch from Overpass API
    info!("Fetching map data from Overpass for bbox {}", bbox.to_overpass_string());
    let overpass_json = match state.overpass.fetch_bbox(&bbox).await {
        Ok(json) => json,
        Err(e) => {
            error!("Overpass API error: {}", e);
            let (status, msg) = match &e {
                MapError::OverpassRequest(_) => {
                    (StatusCode::BAD_GATEWAY, "Failed to reach map data provider")
                }
                MapError::Timeout(_) => (StatusCode::GATEWAY_TIMEOUT, "Map data request timed out"),
                MapError::RateLimited => {
                    (StatusCode::TOO_MANY_REQUESTS, "Rate limited, please retry")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            };
            return (status, Json(json!({ "error": msg }))).into_response();
        }
    };

    // Parse
    let map_data = rmm_map::parse_overpass_json(&overpass_json, bbox);

    // Cache the result (non-fatal if it fails)
    if let Err(e) = state.cache.set(&bbox, &map_data) {
        error!("Failed to cache map data: {}", e);
    }

    // Serialize to MessagePack
    match rmp_serde::to_vec(&map_data) {
        Ok(bytes) => {
            info!(
                "Serving {} bytes of map data ({} roads, {} buildings)",
                bytes.len(),
                map_data.roads.len(),
                map_data.buildings.len()
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/x-msgpack")],
                bytes,
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to serialize map data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
                .into_response()
        }
    }
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/ping", get(ping))
        .route("/api/map", get(get_map))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
