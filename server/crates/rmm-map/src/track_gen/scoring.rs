use std::collections::HashSet;

use super::geo::turn_angle;
use super::types::{CandidateRoute, RoadGraph, ScoreBreakdown};
use crate::LatLon;

/// Weight for turn density component.
const W_TURN_DENSITY: f64 = 0.30;
/// Weight for road variety component.
const W_ROAD_VARIETY: f64 = 0.20;
/// Weight for road width component.
const W_WIDTH: f64 = 0.20;
/// Weight for recovery space component.
const W_RECOVERY: f64 = 0.15;
/// Weight for distinctiveness component.
const W_DISTINCT: f64 = 0.15;

/// Minimum turn angle (radians) to count as a "turn" (~30 degrees).
const MIN_TURN_ANGLE_RAD: f64 = 0.524;

/// Score a candidate route for track quality.
/// `other_routes` is used for distinctiveness scoring (penalize overlap).
pub fn score_route(
    route: &CandidateRoute,
    graph: &RoadGraph,
    other_routes: &[CandidateRoute],
) -> ScoreBreakdown {
    let turn_density = compute_turn_density(route, graph);
    let road_variety = compute_road_variety(route, graph);
    let width = compute_width_score(route, graph);
    let recovery_space = compute_recovery_space(route, graph);
    let distinctiveness = compute_distinctiveness(route, other_routes);

    let total = W_TURN_DENSITY * turn_density
        + W_ROAD_VARIETY * road_variety
        + W_WIDTH * width
        + W_RECOVERY * recovery_space
        + W_DISTINCT * distinctiveness;

    ScoreBreakdown {
        total: total.clamp(0.0, 1.0),
        turn_density,
        road_variety,
        width,
        recovery_space,
        distinctiveness,
    }
}

/// Count turns (angle > 30 degrees) per 100m. Ideal range: 2-5 per 100m → 1.0.
fn compute_turn_density(route: &CandidateRoute, graph: &RoadGraph) -> f64 {
    if route.edge_ids.len() < 2 {
        return 0.0;
    }

    let mut turn_count = 0;

    // Check turn angle at each node junction between consecutive edges
    for window in route.edge_ids.windows(2) {
        let edge_a = &graph.edges[window[0]];
        let edge_b = &graph.edges[window[1]];

        // Get the last two points of edge_a and first two points of edge_b
        if edge_a.geometry.len() >= 2 && edge_b.geometry.len() >= 2 {
            let a_end = edge_a.geometry.len();
            let point_a = LatLon::new(edge_a.geometry[a_end - 2].0, edge_a.geometry[a_end - 2].1);
            let point_b = LatLon::new(edge_a.geometry[a_end - 1].0, edge_a.geometry[a_end - 1].1);
            let point_c = LatLon::new(edge_b.geometry[1].0, edge_b.geometry[1].1);

            let angle = turn_angle(&point_a, &point_b, &point_c).abs();
            if angle > MIN_TURN_ANGLE_RAD {
                turn_count += 1;
            }
        }
    }

    let length_hm = route.total_length_m / 100.0; // hectometers
    if length_hm < 0.01 {
        return 0.0;
    }

    let density = turn_count as f64 / length_hm;

    // Ideal: 2-5 turns per 100m → 1.0. Below or above → diminishing.
    if (2.0..=5.0).contains(&density) {
        1.0
    } else if density < 2.0 {
        (density / 2.0).clamp(0.0, 1.0)
    } else {
        // Too many turns — could be a parking lot
        (1.0 - (density - 5.0) / 5.0).clamp(0.0, 1.0)
    }
}

/// Ratio of unique road types to total edges.
fn compute_road_variety(route: &CandidateRoute, graph: &RoadGraph) -> f64 {
    if route.edge_ids.is_empty() {
        return 0.0;
    }

    let unique_types: HashSet<&str> = route
        .edge_ids
        .iter()
        .map(|&eid| graph.edges[eid].road_type.as_str())
        .collect();

    let variety = unique_types.len() as f64 / route.edge_ids.len() as f64;
    variety.clamp(0.0, 1.0)
}

/// Average road width normalized: (avg_width - 4) / 10, clamped to [0, 1].
fn compute_width_score(route: &CandidateRoute, graph: &RoadGraph) -> f64 {
    if route.edge_ids.is_empty() {
        return 0.0;
    }

    let avg_width: f64 = route
        .edge_ids
        .iter()
        .map(|&eid| graph.edges[eid].width)
        .sum::<f64>()
        / route.edge_ids.len() as f64;

    ((avg_width - 4.0) / 10.0).clamp(0.0, 1.0)
}

/// Fraction of edges with width >= 8m (room to recover from mistakes).
fn compute_recovery_space(route: &CandidateRoute, graph: &RoadGraph) -> f64 {
    if route.edge_ids.is_empty() {
        return 0.0;
    }

    let wide_count = route
        .edge_ids
        .iter()
        .filter(|&&eid| graph.edges[eid].width >= 8.0)
        .count();

    wide_count as f64 / route.edge_ids.len() as f64
}

/// Penalize routes that share >50% of edges with higher-scored routes.
fn compute_distinctiveness(route: &CandidateRoute, others: &[CandidateRoute]) -> f64 {
    if others.is_empty() {
        return 1.0;
    }

    let route_edges: HashSet<usize> = route.edge_ids.iter().copied().collect();
    let mut max_overlap: f64 = 0.0;

    for other in others {
        let other_edges: HashSet<usize> = other.edge_ids.iter().copied().collect();
        let overlap = route_edges.intersection(&other_edges).count() as f64
            / route_edges.len().max(1) as f64;
        max_overlap = max_overlap.max(overlap);
    }

    (1.0 - max_overlap).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track_gen::types::{TrackEdge, TrackMode, TrackNode};

    fn make_route(edge_ids: Vec<usize>, length: f64) -> CandidateRoute {
        CandidateRoute {
            mode: TrackMode::Circuit,
            node_ids: vec![],
            edge_ids,
            total_length_m: length,
            geometry: vec![],
        }
    }

    fn make_graph_with_edges(edges: Vec<TrackEdge>) -> RoadGraph {
        let max_node = edges
            .iter()
            .flat_map(|e| [e.from, e.to])
            .max()
            .unwrap_or(0);
        let mut nodes: Vec<TrackNode> = (0..=max_node)
            .map(|i| TrackNode {
                id: i,
                lat: 51.5,
                lon: 0.0,
                edge_ids: vec![],
            })
            .collect();

        for edge in &edges {
            nodes[edge.from].edge_ids.push(edge.id);
            nodes[edge.to].edge_ids.push(edge.id);
        }

        RoadGraph { nodes, edges }
    }

    #[test]
    fn score_is_between_zero_and_one() {
        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.5, 0.0), (51.505, 0.0)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 500.0,
                road_type: "primary".to_string(),
                width: 10.0,
                geometry: vec![(51.505, 0.0), (51.505, 0.008)],
            },
        ];
        let graph = make_graph_with_edges(edges);
        let route = make_route(vec![0, 1], 1000.0);

        let score = score_route(&route, &graph, &[]);
        assert!(score.total >= 0.0 && score.total <= 1.0);
    }

    #[test]
    fn narrow_roads_reduce_width_score() {
        let edges = vec![TrackEdge {
            id: 0,
            from: 0,
            to: 1,
            length_m: 1000.0,
            road_type: "service".to_string(),
            width: 4.0,
            geometry: vec![(51.5, 0.0), (51.51, 0.0)],
        }];
        let graph = make_graph_with_edges(edges);
        let route = make_route(vec![0], 1000.0);

        let score = score_route(&route, &graph, &[]);
        assert!(
            score.width < 0.1,
            "narrow road width score should be low, was {}",
            score.width
        );
        assert!(
            score.recovery_space < 0.1,
            "narrow road recovery should be low, was {}",
            score.recovery_space
        );
    }

    #[test]
    fn varied_roads_score_higher_variety() {
        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.5, 0.0), (51.505, 0.0)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 500.0,
                road_type: "primary".to_string(),
                width: 10.0,
                geometry: vec![(51.505, 0.0), (51.505, 0.008)],
            },
            TrackEdge {
                id: 2,
                from: 2,
                to: 3,
                length_m: 500.0,
                road_type: "tertiary".to_string(),
                width: 7.0,
                geometry: vec![(51.505, 0.008), (51.5, 0.008)],
            },
        ];
        let graph_varied = make_graph_with_edges(edges);
        let route_varied = make_route(vec![0, 1, 2], 1500.0);

        // All same type
        let edges_same = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.5, 0.0), (51.505, 0.0)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.505, 0.0), (51.505, 0.008)],
            },
            TrackEdge {
                id: 2,
                from: 2,
                to: 3,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.505, 0.008), (51.5, 0.008)],
            },
        ];
        let graph_same = make_graph_with_edges(edges_same);
        let route_same = make_route(vec![0, 1, 2], 1500.0);

        let score_varied = score_route(&route_varied, &graph_varied, &[]);
        let score_same = score_route(&route_same, &graph_same, &[]);

        assert!(
            score_varied.road_variety > score_same.road_variety,
            "varied {} should > same {}",
            score_varied.road_variety,
            score_same.road_variety
        );
    }

    #[test]
    fn distinctiveness_penalizes_overlap() {
        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 500.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![],
            },
        ];
        let graph = make_graph_with_edges(edges);

        let route_a = make_route(vec![0, 1], 1000.0);
        let route_b = make_route(vec![0, 1], 1000.0); // same edges

        let score = score_route(&route_a, &graph, &[route_b]);
        assert!(
            score.distinctiveness < 0.1,
            "overlapping routes should have low distinctiveness, was {}",
            score.distinctiveness
        );
    }
}
