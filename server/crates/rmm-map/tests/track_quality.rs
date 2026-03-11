//! Track quality validation suite.
//!
//! Tests 50 diverse locations to verify >60% produce a raceable route (score > 0.4).
//! These tests hit the live Overpass API and are marked #[ignore].
//! Run with: `cargo test --test track_quality -- --ignored --nocapture`

use rmm_map::{
    build_road_graph, find_circuits, find_sprints,
    track_gen::checkpoints::generate_best_track,
    BBox, OverpassClient,
};
use std::time::Instant;

struct TestLocation {
    name: &'static str,
    category: &'static str,
    bbox: BBox,
}

fn test_locations() -> Vec<TestLocation> {
    vec![
        // Dense urban grids
        TestLocation {
            name: "London - Soho",
            category: "urban_grid",
            bbox: BBox::new(51.510, -0.140, 51.516, -0.130).unwrap(),
        },
        TestLocation {
            name: "NYC - Midtown Manhattan",
            category: "urban_grid",
            bbox: BBox::new(40.752, -73.985, 40.758, -73.975).unwrap(),
        },
        TestLocation {
            name: "Tokyo - Shibuya",
            category: "urban_grid",
            bbox: BBox::new(35.658, 139.698, 35.664, 139.708).unwrap(),
        },
        TestLocation {
            name: "Berlin - Mitte",
            category: "urban_grid",
            bbox: BBox::new(52.518, 13.385, 52.524, 13.395).unwrap(),
        },
        TestLocation {
            name: "Paris - Marais",
            category: "urban_grid",
            bbox: BBox::new(48.855, 2.355, 48.861, 2.365).unwrap(),
        },
        // European winding streets
        TestLocation {
            name: "Rome - Trastevere",
            category: "winding",
            bbox: BBox::new(41.886, 12.466, 41.892, 12.476).unwrap(),
        },
        TestLocation {
            name: "Barcelona - Gothic Quarter",
            category: "winding",
            bbox: BBox::new(41.380, 2.172, 41.386, 2.182).unwrap(),
        },
        TestLocation {
            name: "Prague - Old Town",
            category: "winding",
            bbox: BBox::new(50.085, 14.418, 50.091, 14.428).unwrap(),
        },
        TestLocation {
            name: "Lisbon - Alfama",
            category: "winding",
            bbox: BBox::new(38.710, -9.133, 38.716, -9.123).unwrap(),
        },
        TestLocation {
            name: "Florence - Centro",
            category: "winding",
            bbox: BBox::new(43.769, 11.252, 43.775, 11.262).unwrap(),
        },
        // Suburban areas
        TestLocation {
            name: "London - Croydon suburbs",
            category: "suburban",
            bbox: BBox::new(51.370, -0.105, 51.376, -0.095).unwrap(),
        },
        TestLocation {
            name: "LA - Pasadena suburbs",
            category: "suburban",
            bbox: BBox::new(34.145, -118.155, 34.151, -118.145).unwrap(),
        },
        TestLocation {
            name: "Sydney - Marrickville",
            category: "suburban",
            bbox: BBox::new(-33.915, 151.150, -33.909, 151.160).unwrap(),
        },
        TestLocation {
            name: "Toronto - Scarborough",
            category: "suburban",
            bbox: BBox::new(43.768, -79.260, 43.774, -79.250).unwrap(),
        },
        TestLocation {
            name: "Munich suburbs",
            category: "suburban",
            bbox: BBox::new(48.140, 11.580, 48.146, 11.590).unwrap(),
        },
        // Small towns
        TestLocation {
            name: "Rye, UK",
            category: "small_town",
            bbox: BBox::new(50.948, 0.728, 50.954, 0.738).unwrap(),
        },
        TestLocation {
            name: "Battle, UK",
            category: "small_town",
            bbox: BBox::new(50.912, 0.485, 50.918, 0.495).unwrap(),
        },
        TestLocation {
            name: "Bruges, Belgium",
            category: "small_town",
            bbox: BBox::new(51.206, 3.222, 51.212, 3.232).unwrap(),
        },
        TestLocation {
            name: "Rothenburg, Germany",
            category: "small_town",
            bbox: BBox::new(49.375, 10.175, 49.381, 10.185).unwrap(),
        },
        TestLocation {
            name: "Colmar, France",
            category: "small_town",
            bbox: BBox::new(48.076, 7.354, 48.082, 7.364).unwrap(),
        },
        // Coastal areas
        TestLocation {
            name: "Brighton seafront",
            category: "coastal",
            bbox: BBox::new(50.818, -0.145, 50.824, -0.135).unwrap(),
        },
        TestLocation {
            name: "Nice, France",
            category: "coastal",
            bbox: BBox::new(43.692, 7.265, 43.698, 7.275).unwrap(),
        },
        TestLocation {
            name: "Amalfi, Italy",
            category: "coastal",
            bbox: BBox::new(40.632, 14.598, 40.638, 14.608).unwrap(),
        },
        TestLocation {
            name: "Dubrovnik old town",
            category: "coastal",
            bbox: BBox::new(42.638, 18.106, 42.644, 18.116).unwrap(),
        },
        TestLocation {
            name: "Santorini, Greece",
            category: "coastal",
            bbox: BBox::new(36.415, 25.430, 36.421, 25.440).unwrap(),
        },
        // Grid cities
        TestLocation {
            name: "Chicago - Loop",
            category: "grid",
            bbox: BBox::new(41.878, -87.635, 41.884, -87.625).unwrap(),
        },
        TestLocation {
            name: "Buenos Aires - Palermo",
            category: "grid",
            bbox: BBox::new(-34.590, -58.425, -34.584, -58.415).unwrap(),
        },
        TestLocation {
            name: "Barcelona - Eixample",
            category: "grid",
            bbox: BBox::new(41.388, 2.160, 41.394, 2.170).unwrap(),
        },
        TestLocation {
            name: "Portland - Pearl District",
            category: "grid",
            bbox: BBox::new(45.525, -122.685, 45.531, -122.675).unwrap(),
        },
        TestLocation {
            name: "Melbourne - CBD",
            category: "grid",
            bbox: BBox::new(-37.815, 144.960, -37.809, 144.970).unwrap(),
        },
        // Rural areas (likely to fail)
        TestLocation {
            name: "Scottish Highlands",
            category: "rural",
            bbox: BBox::new(57.250, -5.500, 57.256, -5.490).unwrap(),
        },
        TestLocation {
            name: "Sahara edge",
            category: "rural",
            bbox: BBox::new(31.800, -5.000, 31.806, -4.990).unwrap(),
        },
        TestLocation {
            name: "Mongolian steppe",
            category: "rural",
            bbox: BBox::new(47.900, 106.900, 47.906, 106.910).unwrap(),
        },
        TestLocation {
            name: "Norwegian fjord",
            category: "rural",
            bbox: BBox::new(61.200, 6.800, 61.206, 6.810).unwrap(),
        },
        TestLocation {
            name: "Outback Australia",
            category: "rural",
            bbox: BBox::new(-25.300, 131.000, -25.294, 131.010).unwrap(),
        },
        // Mixed/interesting areas
        TestLocation {
            name: "Amsterdam - canals",
            category: "mixed",
            bbox: BBox::new(52.368, 4.888, 52.374, 4.898).unwrap(),
        },
        TestLocation {
            name: "Venice",
            category: "mixed",
            bbox: BBox::new(45.432, 12.332, 45.438, 12.342).unwrap(),
        },
        TestLocation {
            name: "Singapore - Chinatown",
            category: "mixed",
            bbox: BBox::new(1.278, 103.840, 1.284, 103.850).unwrap(),
        },
        TestLocation {
            name: "Istanbul - Sultanahmet",
            category: "mixed",
            bbox: BBox::new(41.005, 28.975, 41.011, 28.985).unwrap(),
        },
        TestLocation {
            name: "San Francisco - Mission",
            category: "mixed",
            bbox: BBox::new(37.758, -122.422, 37.764, -122.412).unwrap(),
        },
        // User's location
        TestLocation {
            name: "User's town (Ticehurst area)",
            category: "user_local",
            bbox: BBox::new(50.966, 0.243, 50.978, 0.263).unwrap(),
        },
        // More diverse locations
        TestLocation {
            name: "Moscow - Red Square area",
            category: "urban_grid",
            bbox: BBox::new(55.752, 37.615, 55.758, 37.625).unwrap(),
        },
        TestLocation {
            name: "Cairo - Downtown",
            category: "urban_grid",
            bbox: BBox::new(30.045, 31.235, 30.051, 31.245).unwrap(),
        },
        TestLocation {
            name: "Mumbai - Colaba",
            category: "urban_grid",
            bbox: BBox::new(18.920, 72.830, 18.926, 72.840).unwrap(),
        },
        TestLocation {
            name: "São Paulo - Centro",
            category: "urban_grid",
            bbox: BBox::new(-23.548, -46.640, -23.542, -46.630).unwrap(),
        },
        TestLocation {
            name: "Stockholm - Gamla Stan",
            category: "winding",
            bbox: BBox::new(59.323, 18.068, 59.329, 18.078).unwrap(),
        },
        TestLocation {
            name: "Kyoto - Gion",
            category: "winding",
            bbox: BBox::new(35.002, 135.772, 35.008, 135.782).unwrap(),
        },
        TestLocation {
            name: "Edinburgh - Old Town",
            category: "winding",
            bbox: BBox::new(55.948, -3.195, 55.954, -3.185).unwrap(),
        },
        TestLocation {
            name: "Marrakech - Medina",
            category: "winding",
            bbox: BBox::new(31.628, -7.990, 31.634, -7.980).unwrap(),
        },
        TestLocation {
            name: "Hong Kong - Kowloon",
            category: "urban_grid",
            bbox: BBox::new(22.298, 114.168, 22.304, 114.178).unwrap(),
        },
    ]
}

#[ignore]
#[tokio::test]
async fn track_quality_validation() {
    let client = OverpassClient::new();
    let locations = test_locations();
    let total = locations.len();
    let mut successes = 0;
    let mut failures = 0;

    println!("\n{:=<90}", "");
    println!(
        " TRACK QUALITY VALIDATION — {} locations",
        total
    );
    println!("{:=<90}", "");
    println!(
        "{:<35} {:<12} {:<8} {:<8} {:<8} {:<10}",
        "Location", "Category", "Result", "Score", "Mode", "Time(ms)"
    );
    println!("{:-<90}", "");

    for loc in &locations {
        let start = Instant::now();

        // Fetch map data
        let overpass_json = match client.fetch_bbox(&loc.bbox).await {
            Ok(json) => json,
            Err(e) => {
                failures += 1;
                println!(
                    "{:<35} {:<12} {:<8} {:<8} {:<8} {:>10}",
                    loc.name,
                    loc.category,
                    "ERROR",
                    "-",
                    "-",
                    format!("Err: {}", e)
                );
                // Rate limit — wait between requests
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let map_data = rmm_map::parse_overpass_json(&overpass_json, loc.bbox);
        let graph = build_road_graph(&map_data);
        let circuits = find_circuits(&graph);
        let sprints = find_sprints(&graph);

        let elapsed = start.elapsed().as_millis();

        match generate_best_track(circuits, sprints, &graph) {
            Some(track) => {
                let result = if track.score.total > 0.4 {
                    successes += 1;
                    "PASS"
                } else {
                    failures += 1;
                    "LOW"
                };

                let mode = match track.mode {
                    rmm_map::track_gen::types::TrackMode::Circuit => "Circuit",
                    rmm_map::track_gen::types::TrackMode::Sprint => "Sprint",
                };

                println!(
                    "{:<35} {:<12} {:<8} {:<8.2} {:<8} {:>10}",
                    loc.name, loc.category, result, track.score.total, mode, elapsed
                );
            }
            None => {
                failures += 1;
                println!(
                    "{:<35} {:<12} {:<8} {:<8} {:<8} {:>10}",
                    loc.name, loc.category, "NONE", "-", "-", elapsed
                );
            }
        }

        // Rate limit between API calls
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    println!("{:-<90}", "");
    let rate = (successes as f64 / total as f64) * 100.0;
    println!(
        "Results: {} passed, {} failed, {:.1}% success rate",
        successes, failures, rate
    );
    println!("Target: >60% (>= {} of {})", (total as f64 * 0.6).ceil() as usize, total);
    println!("{:=<90}\n", "");

    assert!(
        rate >= 60.0,
        "Track generation success rate {:.1}% is below 60% target",
        rate
    );
}
