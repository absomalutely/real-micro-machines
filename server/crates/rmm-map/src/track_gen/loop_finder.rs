use std::collections::HashSet;

use super::types::{CandidateRoute, RoadGraph, TrackMode};

/// Minimum circuit length in meters.
const MIN_CIRCUIT_LENGTH_M: f64 = 1000.0;
/// Maximum circuit length in meters.
const MAX_CIRCUIT_LENGTH_M: f64 = 3000.0;
/// Maximum number of edges in a circuit.
const MAX_DEPTH: usize = 20;
/// Maximum number of circuits to return.
const MAX_RESULTS: usize = 50;

/// Find circuit (loop) routes in the road graph using bounded DFS.
pub fn find_circuits(graph: &RoadGraph) -> Vec<CandidateRoute> {
    let mut results: Vec<CandidateRoute> = Vec::new();
    let mut seen_edge_sets: Vec<Vec<usize>> = Vec::new();

    // Try starting DFS from each node
    for start_node in 0..graph.nodes.len() {
        if graph.nodes[start_node].edge_ids.is_empty() {
            continue;
        }

        if results.len() >= MAX_RESULTS {
            break;
        }

        let mut path_nodes = vec![start_node];
        let mut path_edges: Vec<usize> = Vec::new();
        let mut path_length = 0.0;
        let mut visited_edges: HashSet<usize> = HashSet::new();

        dfs_find_loops(
            graph,
            start_node,
            &mut path_nodes,
            &mut path_edges,
            &mut path_length,
            &mut visited_edges,
            &mut results,
            &mut seen_edge_sets,
        );
    }

    results
}

#[allow(clippy::too_many_arguments)]
fn dfs_find_loops(
    graph: &RoadGraph,
    start_node: usize,
    path_nodes: &mut Vec<usize>,
    path_edges: &mut Vec<usize>,
    path_length: &mut f64,
    visited_edges: &mut HashSet<usize>,
    results: &mut Vec<CandidateRoute>,
    seen_edge_sets: &mut Vec<Vec<usize>>,
) {
    if results.len() >= MAX_RESULTS {
        return;
    }

    if path_edges.len() >= MAX_DEPTH {
        return;
    }

    let current_node = *path_nodes.last().unwrap();

    for &edge_id in &graph.nodes[current_node].edge_ids {
        if visited_edges.contains(&edge_id) {
            continue;
        }

        let edge = &graph.edges[edge_id];
        let next_node = if edge.from == current_node {
            edge.to
        } else {
            edge.from
        };

        let new_length = *path_length + edge.length_m;

        // Check if we've completed a loop back to start
        if next_node == start_node && path_edges.len() >= 3 {
            if (MIN_CIRCUIT_LENGTH_M..=MAX_CIRCUIT_LENGTH_M).contains(&new_length) {
                // Check for duplicate (same edge set)
                let mut edge_set: Vec<usize> = path_edges.clone();
                edge_set.push(edge_id);
                edge_set.sort();

                let is_duplicate = seen_edge_sets.contains(&edge_set);

                if !is_duplicate {
                    // Build geometry
                    let geometry = build_route_geometry(graph, path_edges, edge_id);
                    let mut node_ids = path_nodes.clone();
                    node_ids.push(start_node);

                    let mut final_edges = path_edges.clone();
                    final_edges.push(edge_id);

                    results.push(CandidateRoute {
                        mode: TrackMode::Circuit,
                        node_ids,
                        edge_ids: final_edges,
                        total_length_m: new_length,
                        geometry,
                    });

                    seen_edge_sets.push(edge_set);
                }
            }
            continue;
        }

        // Don't exceed max length
        if new_length > MAX_CIRCUIT_LENGTH_M {
            continue;
        }

        // Continue DFS
        visited_edges.insert(edge_id);
        path_nodes.push(next_node);
        path_edges.push(edge_id);
        *path_length += edge.length_m;

        dfs_find_loops(
            graph,
            start_node,
            path_nodes,
            path_edges,
            path_length,
            visited_edges,
            results,
            seen_edge_sets,
        );

        *path_length -= edge.length_m;
        path_edges.pop();
        path_nodes.pop();
        visited_edges.remove(&edge_id);
    }
}

/// Build the full geometry polyline for a route (existing path edges + one closing edge).
fn build_route_geometry(
    graph: &RoadGraph,
    path_edges: &[usize],
    closing_edge_id: usize,
) -> Vec<(f64, f64)> {
    let mut geometry = Vec::new();
    let mut all_edges: Vec<usize> = path_edges.to_vec();
    all_edges.push(closing_edge_id);

    for (i, &eid) in all_edges.iter().enumerate() {
        let edge = &graph.edges[eid];
        if i == 0 {
            // Include all points for the first edge
            geometry.extend_from_slice(&edge.geometry);
        } else {
            // Skip the first point (it's the same as the last point of the previous edge)
            if edge.geometry.len() > 1 {
                geometry.extend_from_slice(&edge.geometry[1..]);
            }
        }
    }

    geometry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track_gen::types::{TrackEdge, TrackNode};

    /// Create a simple square graph:
    /// ```
    /// 0 --- 1
    /// |     |
    /// 3 --- 2
    /// ```
    /// Each side is ~555m (0.005 degrees latitude), so the full loop is ~2220m.
    fn make_square_graph() -> RoadGraph {
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
                width: 10.0,
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

        RoadGraph { nodes, edges }
    }

    #[test]
    fn finds_loop_in_square_graph() {
        let graph = make_square_graph();
        let circuits = find_circuits(&graph);
        assert!(!circuits.is_empty(), "should find at least one circuit");

        // All circuits should be the full square (4 edges, ~2220m)
        for circuit in &circuits {
            assert_eq!(circuit.edge_ids.len(), 4);
            assert!(circuit.total_length_m > 1000.0);
            assert!(circuit.total_length_m < 3000.0);
        }
    }

    #[test]
    fn rejects_short_cycles() {
        // Make a tiny square where each side is only 100m
        let nodes = vec![
            TrackNode {
                id: 0,
                lat: 51.500,
                lon: 0.0,
                edge_ids: vec![0, 3],
            },
            TrackNode {
                id: 1,
                lat: 51.500,
                lon: 0.001,
                edge_ids: vec![0, 1],
            },
            TrackNode {
                id: 2,
                lat: 51.4995,
                lon: 0.001,
                edge_ids: vec![1, 2],
            },
            TrackNode {
                id: 3,
                lat: 51.4995,
                lon: 0.0,
                edge_ids: vec![2, 3],
            },
        ];

        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 70.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.500, 0.0), (51.500, 0.001)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 55.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.500, 0.001), (51.4995, 0.001)],
            },
            TrackEdge {
                id: 2,
                from: 2,
                to: 3,
                length_m: 70.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.4995, 0.001), (51.4995, 0.0)],
            },
            TrackEdge {
                id: 3,
                from: 3,
                to: 0,
                length_m: 55.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.4995, 0.0), (51.500, 0.0)],
            },
        ];

        let graph = RoadGraph { nodes, edges };
        let circuits = find_circuits(&graph);
        // Total is 250m, well below 1000m minimum
        assert!(circuits.is_empty(), "should reject short cycles");
    }

    #[test]
    fn deduplicates_same_edge_set() {
        let graph = make_square_graph();
        let circuits = find_circuits(&graph);

        // All circuits with the same 4 edges should be deduplicated
        // Even though DFS starts from different nodes, the edge set {0,1,2,3} is the same
        let unique_edge_sets: Vec<Vec<usize>> = circuits
            .iter()
            .map(|c| {
                let mut es = c.edge_ids.clone();
                es.sort();
                es
            })
            .collect();

        // Check no duplicates
        for i in 0..unique_edge_sets.len() {
            for j in (i + 1)..unique_edge_sets.len() {
                assert_ne!(unique_edge_sets[i], unique_edge_sets[j], "found duplicate");
            }
        }
    }

    #[test]
    fn circuit_has_valid_geometry() {
        let graph = make_square_graph();
        let circuits = find_circuits(&graph);
        assert!(!circuits.is_empty());

        let circuit = &circuits[0];
        assert!(circuit.geometry.len() >= 4, "geometry should have at least 4 points");
    }
}
