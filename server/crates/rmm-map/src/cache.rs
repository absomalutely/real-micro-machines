use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::bbox::BBox;
use crate::error::MapError;
use crate::osm_types::MapData;

const DEFAULT_TTL_SECS: u64 = 7 * 24 * 60 * 60; // 1 week

/// Disk-based cache for MapData using MessagePack serialization.
#[derive(Clone)]
pub struct MapCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl MapCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        Self {
            cache_dir,
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Ensure the cache directory exists.
    pub fn ensure_dir(&self) -> Result<(), MapError> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)?;
            info!("Created cache directory: {}", self.cache_dir.display());
        }
        Ok(())
    }

    /// Generate a deterministic cache key for a bbox.
    fn cache_key(bbox: &BBox) -> String {
        let input = bbox.cache_key_string();
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(&hash[..16]) // 32-char hex string (128 bits, plenty unique)
    }

    /// Path to the cache file for a given bbox.
    fn cache_path(&self, bbox: &BBox) -> PathBuf {
        self.cache_dir.join(format!("{}.msgpack", Self::cache_key(bbox)))
    }

    /// Try to get cached MapData for a bbox. Returns None if not cached or expired.
    pub fn get(&self, bbox: &BBox) -> Option<MapData> {
        let path = self.cache_path(bbox);
        if !path.exists() {
            debug!("Cache miss for bbox {}", bbox.cache_key_string());
            return None;
        }

        // Check TTL
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed > self.ttl {
                        debug!("Cache expired for bbox {}", bbox.cache_key_string());
                        let _ = fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }

        // Deserialize
        match fs::read(&path) {
            Ok(data) => match rmp_serde::from_slice::<MapData>(&data) {
                Ok(map_data) => {
                    info!("Cache hit for bbox {}", bbox.cache_key_string());
                    Some(map_data)
                }
                Err(e) => {
                    warn!("Failed to deserialize cache file {}: {}", path.display(), e);
                    let _ = fs::remove_file(&path);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read cache file {}: {}", path.display(), e);
                None
            }
        }
    }

    /// Store MapData in the cache.
    pub fn set(&self, bbox: &BBox, data: &MapData) -> Result<(), MapError> {
        self.ensure_dir()?;
        let path = self.cache_path(bbox);
        let encoded = rmp_serde::to_vec(data)?;
        fs::write(&path, &encoded)?;
        debug!(
            "Cached {} bytes for bbox {}",
            encoded.len(),
            bbox.cache_key_string()
        );
        Ok(())
    }

    /// Clear all cached data.
    pub fn clear(&self) -> Result<(), MapError> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "msgpack").unwrap_or(false) {
                    fs::remove_file(entry.path())?;
                }
            }
            info!("Cleared cache directory: {}", self.cache_dir.display());
        }
        Ok(())
    }
}

// We need hex encoding for the cache key — add a tiny inline implementation
// to avoid pulling in the `hex` crate just for this.
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osm_types::{Building, LatLon, Road};
    use std::time::Duration;

    fn sample_map_data(bbox: BBox) -> MapData {
        MapData {
            roads: vec![Road {
                id: 1,
                highway_type: "residential".into(),
                points: vec![
                    LatLon::new(51.51, -0.12),
                    LatLon::new(51.52, -0.11),
                ],
                width: 6.0,
                name: Some("Test Road".into()),
                oneway: false,
                lanes: 2,
            }],
            buildings: vec![Building {
                id: 2,
                footprint: vec![
                    LatLon::new(51.511, -0.117),
                    LatLon::new(51.511, -0.116),
                    LatLon::new(51.512, -0.116),
                    LatLon::new(51.511, -0.117),
                ],
                height: 12.0,
            }],
            water: vec![],
            parks: vec![],
            forests: vec![],
            bbox,
        }
    }

    #[test]
    fn write_and_read_cache() {
        let dir = std::env::temp_dir().join("rmm-map-cache-test");
        let _ = fs::remove_dir_all(&dir);

        let cache = MapCache::new(&dir);
        let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        let data = sample_map_data(bbox);

        // Cache miss initially
        assert!(cache.get(&bbox).is_none());

        // Write
        cache.set(&bbox, &data).unwrap();

        // Cache hit
        let cached = cache.get(&bbox).unwrap();
        assert_eq!(cached.roads.len(), 1);
        assert_eq!(cached.buildings.len(), 1);
        assert_eq!(cached.roads[0].name.as_deref(), Some("Test Road"));

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn expired_entries_are_ignored() {
        let dir = std::env::temp_dir().join("rmm-map-cache-ttl-test");
        let _ = fs::remove_dir_all(&dir);

        let cache = MapCache::new(&dir).with_ttl(Duration::from_secs(0));
        let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        let data = sample_map_data(bbox);

        cache.set(&bbox, &data).unwrap();

        // With TTL=0, everything is expired
        std::thread::sleep(Duration::from_millis(10));
        assert!(cache.get(&bbox).is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clear_removes_files() {
        let dir = std::env::temp_dir().join("rmm-map-cache-clear-test");
        let _ = fs::remove_dir_all(&dir);

        let cache = MapCache::new(&dir);
        let bbox = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        let data = sample_map_data(bbox);

        cache.set(&bbox, &data).unwrap();
        assert!(cache.get(&bbox).is_some());

        cache.clear().unwrap();
        assert!(cache.get(&bbox).is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn different_bbox_different_key() {
        let bbox1 = BBox::new(51.510, -0.120, 51.515, -0.110).unwrap();
        let bbox2 = BBox::new(51.511, -0.120, 51.515, -0.110).unwrap();

        let key1 = MapCache::cache_key(&bbox1);
        let key2 = MapCache::cache_key(&bbox2);
        assert_ne!(key1, key2);
    }
}
