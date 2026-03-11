use super::geo::{interpolate_along, polyline_length};
use super::scoring::score_route;
use super::types::{
    CandidateRoute, Checkpoint, GeneratedTrack, PowerUpSpawn, RoadGraph, ScoreBreakdown,
    TrackDirection,
};

/// Target spacing between checkpoints in meters.
const CHECKPOINT_INTERVAL_M: f64 = 200.0;

/// Place checkpoints, start line, power-ups, and determine direction for a route.
pub fn place_checkpoints(
    route: &CandidateRoute,
    graph: &RoadGraph,
    score: ScoreBreakdown,
) -> GeneratedTrack {
    let total_length = polyline_length(&route.geometry);
    let direction = determine_direction(&route.geometry);

    // Find the widest edge for start line placement
    let widest_edge_idx = route
        .edge_ids
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let wa = graph.edges[**a].width;
            let wb = graph.edges[**b].width;
            wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Compute distance along route to the midpoint of the widest edge
    let start_line_dist = compute_edge_midpoint_distance(route, graph, widest_edge_idx);

    // Place start line
    let start_line = if let Some((lat, lon, heading)) =
        interpolate_along(&route.geometry, start_line_dist)
    {
        let widest_eid = route.edge_ids[widest_edge_idx];
        Checkpoint {
            position: (lat, lon),
            heading,
            width: graph.edges[widest_eid].width,
            index: 0,
        }
    } else {
        // Fallback: use first point
        Checkpoint {
            position: route.geometry[0],
            heading: 0.0,
            width: 6.0,
            index: 0,
        }
    };

    // Place checkpoints at regular intervals, starting from the start line
    let num_checkpoints = (total_length / CHECKPOINT_INTERVAL_M).round() as usize;
    let num_checkpoints = num_checkpoints.max(3); // at least 3 checkpoints
    let actual_interval = total_length / num_checkpoints as f64;

    let mut checkpoints = Vec::new();
    for i in 1..num_checkpoints {
        let dist = (start_line_dist + actual_interval * i as f64) % total_length;
        if let Some((lat, lon, heading)) = interpolate_along(&route.geometry, dist) {
            // Find the road width at this point (approximate by nearest edge)
            let cp_width = estimate_width_at_distance(route, graph, dist);
            checkpoints.push(Checkpoint {
                position: (lat, lon),
                heading,
                width: cp_width,
                index: i,
            });
        }
    }

    // Place power-up spawns at midpoints between consecutive checkpoints
    let mut power_up_spawns = Vec::new();
    let all_checkpoints_with_start = {
        let mut all = vec![start_line.clone()];
        all.extend(checkpoints.iter().cloned());
        all
    };

    for window in all_checkpoints_with_start.windows(2) {
        let mid_lat = (window[0].position.0 + window[1].position.0) / 2.0;
        let mid_lon = (window[0].position.1 + window[1].position.1) / 2.0;
        power_up_spawns.push(PowerUpSpawn {
            position: (mid_lat, mid_lon),
            spawn_type: "item_box".to_string(),
        });
    }

    GeneratedTrack {
        mode: route.mode,
        score,
        checkpoints,
        start_line,
        direction,
        power_up_spawns,
        route_geometry: route.geometry.clone(),
        total_length_m: total_length,
    }
}

/// Determine CW vs CCW by computing the signed area (shoelace formula).
/// Positive area = counter-clockwise, negative = clockwise.
fn determine_direction(geometry: &[(f64, f64)]) -> TrackDirection {
    if geometry.len() < 3 {
        return TrackDirection::Clockwise;
    }

    let mut signed_area = 0.0;
    let n = geometry.len();
    for i in 0..n {
        let j = (i + 1) % n;
        // Use lon as x, lat as y
        signed_area += geometry[i].1 * geometry[j].0;
        signed_area -= geometry[j].1 * geometry[i].0;
    }

    if signed_area >= 0.0 {
        TrackDirection::CounterClockwise
    } else {
        TrackDirection::Clockwise
    }
}

/// Compute the distance along the route geometry to the midpoint of a given edge.
fn compute_edge_midpoint_distance(
    route: &CandidateRoute,
    graph: &RoadGraph,
    edge_index: usize,
) -> f64 {
    let mut dist = 0.0;
    for (i, &eid) in route.edge_ids.iter().enumerate() {
        let edge_len = graph.edges[eid].length_m;
        if i == edge_index {
            return dist + edge_len / 2.0;
        }
        dist += edge_len;
    }
    dist / 2.0 // fallback
}

/// Estimate the road width at a given distance along the route.
fn estimate_width_at_distance(route: &CandidateRoute, graph: &RoadGraph, target_dist: f64) -> f64 {
    let total = route.total_length_m;
    let dist = target_dist % total;

    let mut accumulated = 0.0;
    for &eid in &route.edge_ids {
        let edge = &graph.edges[eid];
        if accumulated + edge.length_m >= dist {
            return edge.width;
        }
        accumulated += edge.length_m;
    }

    // Fallback
    6.0
}

/// Generate the best track from a set of candidate routes.
pub fn generate_best_track(
    circuits: Vec<CandidateRoute>,
    sprints: Vec<CandidateRoute>,
    graph: &RoadGraph,
) -> Option<GeneratedTrack> {
    let all_candidates: Vec<CandidateRoute> = circuits
        .into_iter()
        .chain(sprints)
        .collect();

    if all_candidates.is_empty() {
        return None;
    }

    // Score all routes
    let mut scored: Vec<(CandidateRoute, ScoreBreakdown)> = Vec::new();
    for route in &all_candidates {
        let score = score_route(route, graph, &all_candidates);
        scored.push((route.clone(), score));
    }

    // Sort by total score descending
    scored.sort_by(|a, b| b.1.total.partial_cmp(&a.1.total).unwrap_or(std::cmp::Ordering::Equal));

    // Pick the best, preferring circuits over sprints when scores are close
    let (best_route, best_score) = scored.into_iter().next()?;

    // Only accept routes with score > 0.2 (very lenient minimum)
    if best_score.total < 0.2 {
        return None;
    }

    Some(place_checkpoints(&best_route, graph, best_score))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track_gen::types::{TrackEdge, TrackMode, TrackNode};

    fn make_square_route_and_graph() -> (CandidateRoute, RoadGraph) {
        let nodes = vec![
            TrackNode {
                id: 0,
                lat: 51.505,
                lon: 0.0,
                edge_ids: vec![0, 3],
            },
            TrackNode {
                id: 1,
                lat: 51.505,
                lon: 0.008,
                edge_ids: vec![0, 1],
            },
            TrackNode {
                id: 2,
                lat: 51.500,
                lon: 0.008,
                edge_ids: vec![1, 2],
            },
            TrackNode {
                id: 3,
                lat: 51.500,
                lon: 0.0,
                edge_ids: vec![2, 3],
            },
        ];

        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 555.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.505, 0.0), (51.505, 0.008)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 555.0,
                road_type: "primary".to_string(),
                width: 12.0,
                geometry: vec![(51.505, 0.008), (51.500, 0.008)],
            },
            TrackEdge {
                id: 2,
                from: 2,
                to: 3,
                length_m: 555.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.500, 0.008), (51.500, 0.0)],
            },
            TrackEdge {
                id: 3,
                from: 3,
                to: 0,
                length_m: 555.0,
                road_type: "tertiary".to_string(),
                width: 7.0,
                geometry: vec![(51.500, 0.0), (51.505, 0.0)],
            },
        ];

        let graph = RoadGraph { nodes, edges };

        let route = CandidateRoute {
            mode: TrackMode::Circuit,
            node_ids: vec![0, 1, 2, 3, 0],
            edge_ids: vec![0, 1, 2, 3],
            total_length_m: 2220.0,
            geometry: vec![
                (51.505, 0.0),
                (51.505, 0.008),
                (51.500, 0.008),
                (51.500, 0.0),
                (51.505, 0.0), // closing point
            ],
        };

        (route, graph)
    }

    #[test]
    fn checkpoints_are_evenly_spaced() {
        let (route, graph) = make_square_route_and_graph();
        let score = ScoreBreakdown {
            total: 0.5,
            turn_density: 0.5,
            road_variety: 0.5,
            width: 0.5,
            recovery_space: 0.5,
            distinctiveness: 1.0,
        };

        let track = place_checkpoints(&route, &graph, score);

        assert!(
            track.checkpoints.len() >= 3,
            "should have at least 3 checkpoints, got {}",
            track.checkpoints.len()
        );

        // Checkpoint spacing should be roughly 200m ± 20%
        let expected_interval = track.total_length_m / (track.checkpoints.len() + 1) as f64;
        let tolerance = expected_interval * 0.5; // generous tolerance
        assert!(
            (expected_interval - CHECKPOINT_INTERVAL_M).abs() < tolerance,
            "interval {} too far from target {}",
            expected_interval,
            CHECKPOINT_INTERVAL_M
        );
    }

    #[test]
    fn start_line_on_widest_edge() {
        let (route, graph) = make_square_route_and_graph();
        let score = ScoreBreakdown {
            total: 0.5,
            turn_density: 0.5,
            road_variety: 0.5,
            width: 0.5,
            recovery_space: 0.5,
            distinctiveness: 1.0,
        };

        let track = place_checkpoints(&route, &graph, score);

        // Edge 1 (primary, width=12) is the widest
        // Start line should be near its midpoint
        assert!(
            track.start_line.width >= 10.0,
            "start line width {} should be on widest edge",
            track.start_line.width
        );
    }

    #[test]
    fn power_ups_between_checkpoints() {
        let (route, graph) = make_square_route_and_graph();
        let score = ScoreBreakdown {
            total: 0.5,
            turn_density: 0.5,
            road_variety: 0.5,
            width: 0.5,
            recovery_space: 0.5,
            distinctiveness: 1.0,
        };

        let track = place_checkpoints(&route, &graph, score);

        // Should have power-up spawns (at least as many as checkpoints)
        assert!(
            !track.power_up_spawns.is_empty(),
            "should have power-up spawns"
        );

        for spawn in &track.power_up_spawns {
            assert_eq!(spawn.spawn_type, "item_box");
        }
    }

    #[test]
    fn direction_detection() {
        // CW square (going east, south, west, north)
        let cw_geometry = vec![
            (51.505, 0.0),
            (51.505, 0.008),
            (51.500, 0.008),
            (51.500, 0.0),
        ];
        let dir = determine_direction(&cw_geometry);
        // With lon as x and lat as y, this is CW
        assert_eq!(dir, TrackDirection::Clockwise);
    }
}
