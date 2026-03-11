use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("RMM_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
        }
    }
}
