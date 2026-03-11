use std::collections::HashSet;

use super::types::{CandidateRoute, RoadGraph, TrackMode};

/// Minimum sprint length in meters.
const MIN_SPRINT_LENGTH_M: f64 = 1000.0;
/// Maximum sprint length in meters.
const MAX_SPRINT_LENGTH_M: f64 = 3000.0;
/// Maximum start nodes to try.
const MAX_START_NODES: usize = 10;
/// Maximum results to return.
const MAX_RESULTS: usize = 20;

/// Find sprint (point-to-point) routes in the road graph.
/// Used as fallback when no circuit loops are found.
pub fn find_sprints(graph: &RoadGraph) -> Vec<CandidateRoute> {
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        return Vec::new();
    }

    // Prefer starting from endpoint/corner nodes (degree 1 or 2)
    let mut start_candidates: Vec<(usize, usize)> = graph
        .nodes
        .iter()
        .filter(|n| !n.edge_ids.is_empty())
        .map(|n| (n.id, n.edge_ids.len()))
        .collect();

    // Sort by degree ascending (prefer endpoints)
    start_candidates.sort_by_key(|&(_, degree)| degree);

    let mut results: Vec<CandidateRoute> = Vec::new();

    for &(start_node, _) in start_candidates.iter().take(MAX_START_NODES) {
        if results.len() >= MAX_RESULTS {
            break;
        }

        // DFS to find longest path from this start
        let mut best_path: Option<(Vec<usize>, Vec<usize>, f64)> = None;
        let mut path_nodes = vec![start_node];
        let mut path_edges: Vec<usize> = Vec::new();
        let mut visited_edges: HashSet<usize> = HashSet::new();
        let mut path_length = 0.0;

        dfs_longest_path(
            graph,
            &mut path_nodes,
            &mut path_edges,
            &mut path_length,
            &mut visited_edges,
            &mut best_path,
        );

        if let Some((best_nodes, best_edges, best_length)) = best_path {
            if (MIN_SPRINT_LENGTH_M..=MAX_SPRINT_LENGTH_M).contains(&best_length) {
                let geometry = build_sprint_geometry(graph, &best_edges);

                // Check we haven't already found an equivalent route
                let mut edge_set = best_edges.clone();
                edge_set.sort();
                let is_dup = results.iter().any(|r| {
                    let mut es = r.edge_ids.clone();
                    es.sort();
                    es == edge_set
                });

                if !is_dup {
                    results.push(CandidateRoute {
                        mode: TrackMode::Sprint,
                        node_ids: best_nodes,
                        edge_ids: best_edges,
                        total_length_m: best_length,
                        geometry,
                    });
                }
            }
        }
    }

    results
}

fn dfs_longest_path(
    graph: &RoadGraph,
    path_nodes: &mut Vec<usize>,
    path_edges: &mut Vec<usize>,
    path_length: &mut f64,
    visited_edges: &mut HashSet<usize>,
    best: &mut Option<(Vec<usize>, Vec<usize>, f64)>,
) {
    // Update best if current path is longer and within range
    if *path_length >= MIN_SPRINT_LENGTH_M {
        if let Some((_, _, best_len)) = best {
            if *path_length > *best_len && *path_length <= MAX_SPRINT_LENGTH_M {
                *best = Some((path_nodes.clone(), path_edges.clone(), *path_length));
            }
        } else {
            *best = Some((path_nodes.clone(), path_edges.clone(), *path_length));
        }
    }

    if *path_length >= MAX_SPRINT_LENGTH_M {
        return;
    }

    let current = *path_nodes.last().unwrap();

    for &edge_id in &graph.nodes[current].edge_ids {
        if visited_edges.contains(&edge_id) {
            continue;
        }

        let edge = &graph.edges[edge_id];
        let next = if edge.from == current {
            edge.to
        } else {
            edge.from
        };

        visited_edges.insert(edge_id);
        path_nodes.push(next);
        path_edges.push(edge_id);
        *path_length += edge.length_m;

        dfs_longest_path(graph, path_nodes, path_edges, path_length, visited_edges, best);

        *path_length -= edge.length_m;
        path_edges.pop();
        path_nodes.pop();
        visited_edges.remove(&edge_id);
    }
}

fn build_sprint_geometry(graph: &RoadGraph, edges: &[usize]) -> Vec<(f64, f64)> {
    let mut geometry = Vec::new();

    for (i, &eid) in edges.iter().enumerate() {
        let edge = &graph.edges[eid];
        if i == 0 {
            geometry.extend_from_slice(&edge.geometry);
        } else if edge.geometry.len() > 1 {
            geometry.extend_from_slice(&edge.geometry[1..]);
        }
    }

    geometry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track_gen::types::{TrackEdge, TrackNode};

    /// Create a linear graph (tree, no loops):
    /// 0 --- 1 --- 2 --- 3 --- 4
    /// Each segment is ~400m.
    fn make_linear_graph() -> RoadGraph {
        let nodes = vec![
            TrackNode {
                id: 0,
                lat: 51.500,
                lon: 0.0,
                edge_ids: vec![0],
            },
            TrackNode {
                id: 1,
                lat: 51.500,
                lon: 0.006,
                edge_ids: vec![0, 1],
            },
            TrackNode {
                id: 2,
                lat: 51.500,
                lon: 0.012,
                edge_ids: vec![1, 2],
            },
            TrackNode {
                id: 3,
                lat: 51.500,
                lon: 0.018,
                edge_ids: vec![2, 3],
            },
            TrackNode {
                id: 4,
                lat: 51.500,
                lon: 0.024,
                edge_ids: vec![3],
            },
        ];

        let edges = vec![
            TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 400.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.500, 0.0), (51.500, 0.006)],
            },
            TrackEdge {
                id: 1,
                from: 1,
                to: 2,
                length_m: 400.0,
                road_type: "primary".to_string(),
                width: 10.0,
                geometry: vec![(51.500, 0.006), (51.500, 0.012)],
            },
            TrackEdge {
                id: 2,
                from: 2,
                to: 3,
                length_m: 400.0,
                road_type: "tertiary".to_string(),
                width: 7.0,
                geometry: vec![(51.500, 0.012), (51.500, 0.018)],
            },
            TrackEdge {
                id: 3,
                from: 3,
                to: 4,
                length_m: 400.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.500, 0.018), (51.500, 0.024)],
            },
        ];

        RoadGraph { nodes, edges }
    }

    #[test]
    fn finds_sprint_in_linear_graph() {
        let graph = make_linear_graph();
        let sprints = find_sprints(&graph);
        assert!(!sprints.is_empty(), "should find at least one sprint");

        for sprint in &sprints {
            assert_eq!(sprint.mode, TrackMode::Sprint);
            assert!(sprint.total_length_m >= MIN_SPRINT_LENGTH_M);
            assert!(sprint.total_length_m <= MAX_SPRINT_LENGTH_M);
        }
    }

    #[test]
    fn sprint_does_not_revisit_edges() {
        let graph = make_linear_graph();
        let sprints = find_sprints(&graph);

        for sprint in &sprints {
            let mut seen = HashSet::new();
            for &eid in &sprint.edge_ids {
                assert!(seen.insert(eid), "edge {eid} visited twice");
            }
        }
    }

    #[test]
    fn empty_graph_returns_no_sprints() {
        let graph = RoadGraph {
            nodes: vec![],
            edges: vec![],
        };
        let sprints = find_sprints(&graph);
        assert!(sprints.is_empty());
    }

    #[test]
    fn too_short_graph_returns_no_sprints() {
        // Single short edge (100m)
        let graph = RoadGraph {
            nodes: vec![
                TrackNode {
                    id: 0,
                    lat: 51.5,
                    lon: 0.0,
                    edge_ids: vec![0],
                },
                TrackNode {
                    id: 1,
                    lat: 51.5,
                    lon: 0.001,
                    edge_ids: vec![0],
                },
            ],
            edges: vec![TrackEdge {
                id: 0,
                from: 0,
                to: 1,
                length_m: 100.0,
                road_type: "residential".to_string(),
                width: 6.0,
                geometry: vec![(51.5, 0.0), (51.5, 0.001)],
            }],
        };
        let sprints = find_sprints(&graph);
        assert!(sprints.is_empty());
    }
}
