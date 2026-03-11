use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub cache_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("RMM_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
            cache_dir: env::var("RMM_CACHE_DIR")
                .unwrap_or_else(|_| ".cache/maps".to_string()),
        }
    }
}
