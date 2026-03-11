use serde::{Deserialize, Serialize};

/// A graph representation of the road network, with nodes (intersections)
/// and edges (road segments between intersections).
#[derive(Debug, Clone)]
pub struct RoadGraph {
    pub nodes: Vec<TrackNode>,
    pub edges: Vec<TrackEdge>,
}

/// A node in the road graph, representing an intersection or endpoint.
#[derive(Debug, Clone)]
pub struct TrackNode {
    pub id: usize,
    pub lat: f64,
    pub lon: f64,
    /// Indices into the parent graph's edges vec.
    pub edge_ids: Vec<usize>,
}

/// An edge in the road graph, representing a road segment between two nodes.
#[derive(Debug, Clone)]
pub struct TrackEdge {
    pub id: usize,
    /// Index of the source node.
    pub from: usize,
    /// Index of the destination node.
    pub to: usize,
    /// Length in meters (sum of haversine distances along geometry).
    pub length_m: f64,
    /// OSM highway type (e.g. "residential", "primary").
    pub road_type: String,
    /// Road width in meters.
    pub width: f64,
    /// Intermediate lat/lon points forming the edge geometry.
    pub geometry: Vec<(f64, f64)>,
}

/// Whether the track is a circuit (loop) or sprint (point-to-point).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackMode {
    Circuit,
    Sprint,
}

/// A candidate route through the road graph, before checkpoint placement.
#[derive(Debug, Clone)]
pub struct CandidateRoute {
    pub mode: TrackMode,
    /// Ordered node indices forming the route.
    pub node_ids: Vec<usize>,
    /// Ordered edge indices forming the route.
    pub edge_ids: Vec<usize>,
    /// Total route length in meters.
    pub total_length_m: f64,
    /// Full polyline (lat, lon) for rendering.
    pub geometry: Vec<(f64, f64)>,
}

/// Breakdown of track quality scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Weighted total score, 0.0–1.0.
    pub total: f64,
    /// Turns per 100m, weight 0.3.
    pub turn_density: f64,
    /// Unique road types / total edges, weight 0.2.
    pub road_variety: f64,
    /// Average road width normalized, weight 0.2.
    pub width: f64,
    /// Fraction of edges with width >= 8m, weight 0.15.
    pub recovery_space: f64,
    /// Low overlap with other candidates, weight 0.15.
    pub distinctiveness: f64,
}

/// Direction of travel around a circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackDirection {
    Clockwise,
    CounterClockwise,
}

/// A checkpoint gate on the track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// (lat, lon) position of the checkpoint center.
    pub position: (f64, f64),
    /// Direction of travel in radians.
    pub heading: f64,
    /// Width of the checkpoint gate in meters.
    pub width: f64,
    /// Checkpoint index (0-based, start line is separate).
    pub index: usize,
}

/// A power-up spawn point on the track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerUpSpawn {
    /// (lat, lon) position.
    pub position: (f64, f64),
    /// Type of spawn (e.g. "item_box").
    pub spawn_type: String,
}

/// A fully generated track with route, checkpoints, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTrack {
    pub mode: TrackMode,
    /// Quality score breakdown.
    pub score: ScoreBreakdown,
    /// Ordered checkpoints along the track.
    pub checkpoints: Vec<Checkpoint>,
    /// The start/finish line checkpoint.
    pub start_line: Checkpoint,
    /// Direction of travel (for circuits).
    pub direction: TrackDirection,
    /// Power-up spawn locations.
    pub power_up_spawns: Vec<PowerUpSpawn>,
    /// Full route geometry as (lat, lon) pairs.
    pub route_geometry: Vec<(f64, f64)>,
    /// Total track length in meters.
    pub total_length_m: f64,
}
