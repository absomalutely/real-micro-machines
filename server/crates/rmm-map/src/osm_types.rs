use serde::{Deserialize, Serialize};

use crate::bbox::BBox;

/// All map data for a bounding box region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub roads: Vec<Road>,
    pub buildings: Vec<Building>,
    pub water: Vec<Polygon>,
    pub parks: Vec<Polygon>,
    pub forests: Vec<Polygon>,
    pub bbox: BBox,
}

/// A road segment parsed from OSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub id: i64,
    pub highway_type: String,
    pub points: Vec<LatLon>,
    /// Estimated road width in meters based on highway type.
    pub width: f64,
    pub name: Option<String>,
    pub oneway: bool,
    pub lanes: u8,
}

/// A building parsed from OSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: i64,
    pub footprint: Vec<LatLon>,
    /// Estimated height in meters (from height tag, levels*3, or default 8m).
    pub height: f64,
}

/// A generic polygon (water, park, forest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub id: i64,
    pub points: Vec<LatLon>,
    pub polygon_type: String,
}

/// A geographic coordinate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl LatLon {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}
