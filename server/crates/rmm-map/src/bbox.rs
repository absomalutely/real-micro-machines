use serde::{Deserialize, Serialize};

use crate::error::MapError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BBox {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl BBox {
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Result<Self, MapError> {
        let bbox = Self {
            south,
            west,
            north,
            east,
        };
        bbox.validate()?;
        Ok(bbox)
    }

    /// Validate that the bounding box is well-formed.
    pub fn validate(&self) -> Result<(), MapError> {
        if self.south >= self.north {
            return Err(MapError::InvalidBBox(
                "south must be less than north".into(),
            ));
        }
        if self.west >= self.east {
            return Err(MapError::InvalidBBox(
                "west must be less than east".into(),
            ));
        }
        if self.south < -90.0 || self.north > 90.0 {
            return Err(MapError::InvalidBBox(
                "latitude must be between -90 and 90".into(),
            ));
        }
        if self.west < -180.0 || self.east > 180.0 {
            return Err(MapError::InvalidBBox(
                "longitude must be between -180 and 180".into(),
            ));
        }
        Ok(())
    }

    /// Approximate side lengths in meters using Haversine-like estimation.
    pub fn side_lengths_m(&self) -> (f64, f64) {
        let center_lat = (self.south + self.north) / 2.0;
        let lat_rad = center_lat.to_radians();

        let ns_meters = (self.north - self.south) * 111_320.0;
        let ew_meters = (self.east - self.west) * 111_320.0 * lat_rad.cos();

        (ns_meters, ew_meters)
    }

    /// Check that both sides are within the given range (inclusive).
    pub fn validate_size(&self, min_m: f64, max_m: f64) -> Result<(), MapError> {
        let (ns, ew) = self.side_lengths_m();
        if ns < min_m || ew < min_m {
            return Err(MapError::InvalidBBox(format!(
                "bounding box too small: {:.0}m x {:.0}m (minimum {}m)",
                ns, ew, min_m
            )));
        }
        if ns > max_m || ew > max_m {
            return Err(MapError::InvalidBBox(format!(
                "bounding box too large: {:.0}m x {:.0}m (maximum {}m)",
                ns, ew, max_m
            )));
        }
        Ok(())
    }

    /// Center point of the bounding box.
    pub fn center(&self) -> (f64, f64) {
        (
            (self.south + self.north) / 2.0,
            (self.west + self.east) / 2.0,
        )
    }

    /// Format as Overpass API bbox string: "south,west,north,east"
    pub fn to_overpass_string(&self) -> String {
        format!("{},{},{},{}", self.south, self.west, self.north, self.east)
    }

    /// Cache key string (6 decimal precision).
    pub fn cache_key_string(&self) -> String {
        format!(
            "{:.6},{:.6},{:.6},{:.6}",
            self.south, self.west, self.north, self.east
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bbox() {
        let bbox = BBox::new(51.51, -0.12, 51.52, -0.11).unwrap();
        assert!((bbox.south - 51.51).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_south_north() {
        let result = BBox::new(51.52, -0.12, 51.51, -0.11);
        assert!(result.is_err());
    }

    #[test]
    fn side_lengths_reasonable() {
        let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        let (ns, ew) = bbox.side_lengths_m();
        // ~555m north-south, ~695m east-west at London latitude
        assert!(ns > 500.0 && ns < 600.0);
        assert!(ew > 600.0 && ew < 800.0);
    }

    #[test]
    fn size_validation() {
        let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        assert!(bbox.validate_size(100.0, 2000.0).is_ok());
        assert!(bbox.validate_size(1000.0, 2000.0).is_err()); // too small
    }

    #[test]
    fn overpass_string_format() {
        let bbox = BBox::new(51.51, -0.12, 51.52, -0.11).unwrap();
        assert_eq!(bbox.to_overpass_string(), "51.51,-0.12,51.52,-0.11");
    }
}
