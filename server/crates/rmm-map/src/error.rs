use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapError {
    #[error("Overpass API request failed: {0}")]
    OverpassRequest(#[from] reqwest::Error),

    #[error("Overpass API returned error: {0}")]
    OverpassResponse(String),

    #[error("Failed to parse Overpass JSON: {0}")]
    ParseError(String),

    #[error("Invalid bounding box: {0}")]
    InvalidBBox(String),

    #[error("Cache I/O error: {0}")]
    CacheError(#[from] std::io::Error),

    #[error("MessagePack encode error: {0}")]
    MsgpackEncode(#[from] rmp_serde::encode::Error),

    #[error("MessagePack decode error: {0}")]
    MsgpackDecode(#[from] rmp_serde::decode::Error),

    #[error("Rate limited: please wait before making another request")]
    RateLimited,

    #[error("Request timed out after {0} seconds")]
    Timeout(u64),
}
