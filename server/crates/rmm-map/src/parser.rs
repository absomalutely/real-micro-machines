use serde_json::Value;
use tracing::debug;

use crate::bbox::BBox;
use crate::osm_types::{Building, LatLon, MapData, Polygon, Road};

/// Road width lookup table (meters) by OSM highway type.
fn road_width(highway_type: &str) -> f64 {
    match highway_type {
        "motorway" | "motorway_link" => 14.0,
        "trunk" | "trunk_link" => 12.0,
        "primary" | "primary_link" => 10.0,
        "secondary" | "secondary_link" => 8.0,
        "tertiary" | "tertiary_link" => 7.0,
        "residential" | "living_street" | "unclassified" => 6.0,
        "service" => 4.0,
        "footway" | "cycleway" | "path" | "pedestrian" => 2.0,
        _ => 6.0, // default
    }
}

/// Extract geometry points from an Overpass element's `geometry` array.
fn extract_geometry(element: &Value) -> Vec<LatLon> {
    element
        .get("geometry")
        .and_then(|g| g.as_array())
        .map(|coords| {
            coords
                .iter()
                .filter_map(|c| {
                    let lat = c.get("lat")?.as_f64()?;
                    let lon = c.get("lon")?.as_f64()?;
                    Some(LatLon::new(lat, lon))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get a string tag value from an element's tags object.
fn get_tag<'a>(element: &'a Value, key: &str) -> Option<&'a str> {
    element.get("tags")?.get(key)?.as_str()
}

/// Get a numeric tag value (tries parsing the string).
fn get_tag_f64(element: &Value, key: &str) -> Option<f64> {
    let s = get_tag(element, key)?;
    s.parse::<f64>().ok()
}

/// Estimate building height from OSM tags.
fn building_height(element: &Value) -> f64 {
    // Direct height tag
    if let Some(h) = get_tag_f64(element, "height") {
        return h;
    }
    // building:levels * 3 meters
    if let Some(levels) = get_tag_f64(element, "building:levels") {
        return levels * 3.0;
    }
    // Default
    8.0
}

/// Parse Overpass JSON response into typed MapData.
pub fn parse_overpass_json(json: &Value, bbox: BBox) -> MapData {
    let elements = json
        .get("elements")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();

    let mut roads = Vec::new();
    let mut buildings = Vec::new();
    let mut water = Vec::new();
    let mut parks = Vec::new();
    let mut forests = Vec::new();

    for element in &elements {
        let id = element.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        let tags = element.get("tags");

        if tags.is_none() {
            continue;
        }

        let points = extract_geometry(element);

        // Classify by tags
        if let Some(highway) = get_tag(element, "highway") {
            if points.len() >= 2 {
                let oneway = get_tag(element, "oneway")
                    .map(|v| v == "yes" || v == "1" || v == "true")
                    .unwrap_or(false);
                let lanes = get_tag(element, "lanes")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(2);

                roads.push(Road {
                    id,
                    highway_type: highway.to_string(),
                    width: road_width(highway),
                    name: get_tag(element, "name").map(String::from),
                    oneway,
                    lanes,
                    points,
                });
            }
        } else if get_tag(element, "building").is_some() {
            if points.len() >= 3 {
                buildings.push(Building {
                    id,
                    height: building_height(element),
                    footprint: points,
                });
            }
        } else if get_tag(element, "natural") == Some("water") {
            if points.len() >= 3 {
                water.push(Polygon {
                    id,
                    polygon_type: "water".to_string(),
                    points,
                });
            }
        } else if get_tag(element, "leisure") == Some("park") {
            if points.len() >= 3 {
                parks.push(Polygon {
                    id,
                    polygon_type: "park".to_string(),
                    points,
                });
            }
        } else if get_tag(element, "landuse") == Some("forest")
            && points.len() >= 3 {
                forests.push(Polygon {
                    id,
                    polygon_type: "forest".to_string(),
                    points,
                });
            }
    }

    debug!(
        "Parsed: {} roads, {} buildings, {} water, {} parks, {} forests",
        roads.len(),
        buildings.len(),
        water.len(),
        parks.len(),
        forests.len()
    );

    MapData {
        roads,
        buildings,
        water,
        parks,
        forests,
        bbox,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_overpass_json() -> Value {
        json!({
            "elements": [
                {
                    "type": "way",
                    "id": 1001,
                    "tags": {
                        "highway": "residential",
                        "name": "Test Street",
                        "lanes": "2"
                    },
                    "geometry": [
                        {"lat": 51.511, "lon": -0.116},
                        {"lat": 51.512, "lon": -0.115},
                        {"lat": 51.513, "lon": -0.114}
                    ]
                },
                {
                    "type": "way",
                    "id": 1002,
                    "tags": {
                        "highway": "primary",
                        "name": "Main Road",
                        "oneway": "yes"
                    },
                    "geometry": [
                        {"lat": 51.510, "lon": -0.120},
                        {"lat": 51.510, "lon": -0.118}
                    ]
                },
                {
                    "type": "way",
                    "id": 2001,
                    "tags": {
                        "building": "yes",
                        "building:levels": "4"
                    },
                    "geometry": [
                        {"lat": 51.511, "lon": -0.117},
                        {"lat": 51.511, "lon": -0.116},
                        {"lat": 51.512, "lon": -0.116},
                        {"lat": 51.512, "lon": -0.117},
                        {"lat": 51.511, "lon": -0.117}
                    ]
                },
                {
                    "type": "way",
                    "id": 2002,
                    "tags": {
                        "building": "residential",
                        "height": "15"
                    },
                    "geometry": [
                        {"lat": 51.513, "lon": -0.115},
                        {"lat": 51.513, "lon": -0.114},
                        {"lat": 51.514, "lon": -0.114},
                        {"lat": 51.513, "lon": -0.115}
                    ]
                },
                {
                    "type": "way",
                    "id": 3001,
                    "tags": {
                        "natural": "water"
                    },
                    "geometry": [
                        {"lat": 51.510, "lon": -0.115},
                        {"lat": 51.510, "lon": -0.113},
                        {"lat": 51.511, "lon": -0.113},
                        {"lat": 51.510, "lon": -0.115}
                    ]
                },
                {
                    "type": "way",
                    "id": 4001,
                    "tags": {
                        "leisure": "park"
                    },
                    "geometry": [
                        {"lat": 51.514, "lon": -0.118},
                        {"lat": 51.514, "lon": -0.116},
                        {"lat": 51.515, "lon": -0.116},
                        {"lat": 51.514, "lon": -0.118}
                    ]
                },
                {
                    "type": "way",
                    "id": 5001,
                    "tags": {
                        "landuse": "forest"
                    },
                    "geometry": [
                        {"lat": 51.515, "lon": -0.120},
                        {"lat": 51.515, "lon": -0.118},
                        {"lat": 51.516, "lon": -0.118},
                        {"lat": 51.515, "lon": -0.120}
                    ]
                }
            ]
        })
    }

    #[test]
    fn parses_roads() {
        let bbox = BBox::new(51.510, -0.120, 51.516, -0.110).unwrap();
        let data = parse_overpass_json(&sample_overpass_json(), bbox);
        assert_eq!(data.roads.len(), 2);

        let residential = data.roads.iter().find(|r| r.id == 1001).unwrap();
        assert_eq!(residential.highway_type, "residential");
        assert_eq!(residential.width, 6.0);
        assert_eq!(residential.name.as_deref(), Some("Test Street"));
        assert!(!residential.oneway);
        assert_eq!(residential.lanes, 2);
        assert_eq!(residential.points.len(), 3);

        let primary = data.roads.iter().find(|r| r.id == 1002).unwrap();
        assert_eq!(primary.highway_type, "primary");
        assert_eq!(primary.width, 10.0);
        assert!(primary.oneway);
    }

    #[test]
    fn parses_buildings() {
        let bbox = BBox::new(51.510, -0.120, 51.516, -0.110).unwrap();
        let data = parse_overpass_json(&sample_overpass_json(), bbox);
        assert_eq!(data.buildings.len(), 2);

        let b1 = data.buildings.iter().find(|b| b.id == 2001).unwrap();
        assert_eq!(b1.height, 12.0); // 4 levels * 3m

        let b2 = data.buildings.iter().find(|b| b.id == 2002).unwrap();
        assert_eq!(b2.height, 15.0); // explicit height tag
    }

    #[test]
    fn parses_water_parks_forests() {
        let bbox = BBox::new(51.510, -0.120, 51.516, -0.110).unwrap();
        let data = parse_overpass_json(&sample_overpass_json(), bbox);
        assert_eq!(data.water.len(), 1);
        assert_eq!(data.parks.len(), 1);
        assert_eq!(data.forests.len(), 1);
    }

    #[test]
    fn road_width_lookup() {
        assert_eq!(road_width("motorway"), 14.0);
        assert_eq!(road_width("primary"), 10.0);
        assert_eq!(road_width("residential"), 6.0);
        assert_eq!(road_width("footway"), 2.0);
        assert_eq!(road_width("unknown_type"), 6.0);
    }

    #[test]
    fn empty_elements() {
        let json = json!({"elements": []});
        let bbox = BBox::new(51.51, -0.12, 51.52, -0.11).unwrap();
        let data = parse_overpass_json(&json, bbox);
        assert!(data.roads.is_empty());
        assert!(data.buildings.is_empty());
    }
}
