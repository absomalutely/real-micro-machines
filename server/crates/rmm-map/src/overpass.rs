use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::bbox::BBox;
use crate::error::MapError;

const OVERPASS_API_URL: &str = "https://overpass-api.de/api/interpreter";
const REQUEST_TIMEOUT_SECS: u64 = 30;
const RATE_LIMIT_DELAY_SECS: u64 = 2;

/// Client for fetching map data from the Overpass API.
#[derive(Clone)]
pub struct OverpassClient {
    client: Client,
    /// Semaphore to enforce rate limiting (max 1 concurrent request).
    semaphore: Arc<Semaphore>,
}

impl OverpassClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    /// Build the Overpass QL query for a bounding box.
    pub fn build_query(bbox: &BBox) -> String {
        let bbox_str = bbox.to_overpass_string();
        format!(
            r#"[out:json][timeout:30];
(
  way["highway"]({bbox});
  way["building"]({bbox});
  way["natural"="water"]({bbox});
  relation["natural"="water"]({bbox});
  way["leisure"="park"]({bbox});
  relation["leisure"="park"]({bbox});
  way["landuse"="forest"]({bbox});
  relation["landuse"="forest"]({bbox});
);
out body geom;"#,
            bbox = bbox_str
        )
    }

    /// Fetch raw Overpass JSON for a bounding box.
    pub async fn fetch_bbox(
        &self,
        bbox: &BBox,
    ) -> Result<serde_json::Value, MapError> {
        // Acquire semaphore for rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MapError::RateLimited)?;

        let query = Self::build_query(bbox);
        debug!("Overpass query for bbox {}: {} bytes", bbox.to_overpass_string(), query.len());

        let response = self
            .client
            .post(OVERPASS_API_URL)
            .form(&[("data", &query)])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            warn!("Overpass API returned {}: {}", status, &body[..body.len().min(200)]);
            return Err(MapError::OverpassResponse(format!(
                "HTTP {}: {}",
                status,
                &body[..body.len().min(200)]
            )));
        }

        let json: serde_json::Value = response.json().await?;

        // Check for Overpass-level errors
        if let Some(remark) = json.get("remark") {
            warn!("Overpass API remark: {}", remark);
            return Err(MapError::OverpassResponse(
                remark.as_str().unwrap_or("unknown error").to_string(),
            ));
        }

        let element_count = json
            .get("elements")
            .and_then(|e| e.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        info!("Overpass returned {} elements for bbox {}", element_count, bbox.to_overpass_string());

        // Rate limit delay after successful request
        tokio::time::sleep(Duration::from_secs(RATE_LIMIT_DELAY_SECS)).await;

        Ok(json)
    }
}

impl Default for OverpassClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_contains_bbox() {
        let bbox = BBox::new(51.51, -0.12, 51.52, -0.11).unwrap();
        let query = OverpassClient::build_query(&bbox);
        assert!(query.contains("51.51,-0.12,51.52,-0.11"));
        assert!(query.contains(r#"way["highway"]"#));
        assert!(query.contains(r#"way["building"]"#));
        assert!(query.contains(r#"way["natural"="water"]"#));
        assert!(query.contains(r#"way["leisure"="park"]"#));
        assert!(query.contains(r#"way["landuse"="forest"]"#));
        assert!(query.contains("out body geom;"));
    }

    #[test]
    fn query_format_is_valid_overpass_ql() {
        let bbox = BBox::new(51.51, -0.12, 51.52, -0.11).unwrap();
        let query = OverpassClient::build_query(&bbox);
        assert!(query.starts_with("[out:json]"));
        assert!(query.ends_with("out body geom;"));
    }
}
