use std::f64::consts::PI;

use crate::LatLon;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Compute the haversine distance between two points in meters.
pub fn haversine(a: &LatLon, b: &LatLon) -> f64 {
    let d_lat = (b.lat - a.lat).to_radians();
    let d_lon = (b.lon - a.lon).to_radians();
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();

    let h = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().asin()
}

/// Compute the total length of a polyline in meters.
pub fn polyline_length(points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|w| {
            haversine(
                &LatLon::new(w[0].0, w[0].1),
                &LatLon::new(w[1].0, w[1].1),
            )
        })
        .sum()
}

/// Compute the signed turn angle at point b, going from a→b→c.
/// Returns radians in [-PI, PI]. Positive = left turn, negative = right turn.
pub fn turn_angle(a: &LatLon, b: &LatLon, c: &LatLon) -> f64 {
    let bearing_ab = bearing(a, b);
    let bearing_bc = bearing(b, c);
    let mut angle = bearing_bc - bearing_ab;

    // Normalize to [-PI, PI]
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// Compute the initial bearing from point a to point b in radians [0, 2*PI).
pub fn bearing(a: &LatLon, b: &LatLon) -> f64 {
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let d_lon = (b.lon - a.lon).to_radians();

    let y = d_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
    y.atan2(x).rem_euclid(2.0 * PI)
}

/// Interpolate a point along a polyline at a given distance from the start.
/// Returns (lat, lon, heading) where heading is the bearing at that point.
pub fn interpolate_along(points: &[(f64, f64)], target_dist: f64) -> Option<(f64, f64, f64)> {
    if points.len() < 2 {
        return None;
    }

    let mut accumulated = 0.0;
    for w in points.windows(2) {
        let a = LatLon::new(w[0].0, w[0].1);
        let b = LatLon::new(w[1].0, w[1].1);
        let seg_len = haversine(&a, &b);

        if accumulated + seg_len >= target_dist {
            let t = if seg_len > 0.0 {
                (target_dist - accumulated) / seg_len
            } else {
                0.0
            };
            let lat = a.lat + t * (b.lat - a.lat);
            let lon = a.lon + t * (b.lon - a.lon);
            let heading = bearing(&a, &b);
            return Some((lat, lon, heading));
        }
        accumulated += seg_len;
    }

    // Past the end — return last point with bearing from second-to-last
    let n = points.len();
    let a = LatLon::new(points[n - 2].0, points[n - 2].1);
    let b = LatLon::new(points[n - 1].0, points[n - 1].1);
    Some((b.lat, b.lon, bearing(&a, &b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_same_point_is_zero() {
        let p = LatLon::new(51.5, -0.1);
        assert!((haversine(&p, &p)).abs() < 0.01);
    }

    #[test]
    fn haversine_known_distance() {
        // London to Paris ~ 343 km
        let london = LatLon::new(51.5074, -0.1278);
        let paris = LatLon::new(48.8566, 2.3522);
        let dist = haversine(&london, &paris);
        assert!((dist - 343_500.0).abs() < 5_000.0, "dist was {dist}");
    }

    #[test]
    fn haversine_short_distance() {
        // Two points ~111m apart (0.001 degree latitude)
        let a = LatLon::new(51.500, 0.0);
        let b = LatLon::new(51.501, 0.0);
        let dist = haversine(&a, &b);
        assert!((dist - 111.0).abs() < 2.0, "dist was {dist}");
    }

    #[test]
    fn polyline_length_basic() {
        let pts = vec![(51.500, 0.0), (51.501, 0.0), (51.502, 0.0)];
        let len = polyline_length(&pts);
        assert!((len - 222.0).abs() < 5.0, "len was {len}");
    }

    #[test]
    fn turn_angle_straight() {
        let a = LatLon::new(51.500, 0.0);
        let b = LatLon::new(51.501, 0.0);
        let c = LatLon::new(51.502, 0.0);
        let angle = turn_angle(&a, &b, &c);
        assert!(angle.abs() < 0.01, "angle was {angle}");
    }

    #[test]
    fn turn_angle_right_turn() {
        // Going north, then turning east
        let a = LatLon::new(51.500, 0.0);
        let b = LatLon::new(51.501, 0.0);
        let c = LatLon::new(51.501, 0.001);
        let angle = turn_angle(&a, &b, &c);
        // Right turn: bearing changes from ~0 (north) to ~π/2 (east), so angle ≈ +π/2
        assert!(angle.abs() > 1.0 && angle.abs() < 2.0, "angle was {angle}");
    }

    #[test]
    fn interpolate_along_midpoint() {
        let pts = vec![(51.500, 0.0), (51.502, 0.0)];
        let total = polyline_length(&pts);
        let result = interpolate_along(&pts, total / 2.0).unwrap();
        assert!(
            (result.0 - 51.501).abs() < 0.0001,
            "lat was {}",
            result.0
        );
    }

    #[test]
    fn interpolate_along_start() {
        let pts = vec![(51.500, 0.0), (51.502, 0.0)];
        let result = interpolate_along(&pts, 0.0).unwrap();
        assert!(
            (result.0 - 51.500).abs() < 0.0001,
            "lat was {}",
            result.0
        );
    }
}
