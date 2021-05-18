use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::fmt;
use std::time::Duration;
use log::error;

const CONFIG_FILE_PATH: &str = "./config/Default.toml";
const CONFIG_FILE_PREFIX: &str = "./config/";

fn default_duration() -> Duration {
    return Duration::from_secs(5);
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    #[serde(rename = "type")]
    pub db_type: DatabaseType,
    pub url: String,
    pub batching: bool,
    pub batching_frequency: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Maxmind {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub maxmind: Option<Maxmind>,
    pub server: Server,
    pub database: Database,
    pub log: Log,
    pub env: ENV,
}

#[derive(Clone, Debug, Deserialize)]
pub enum DatabaseType {
    ClickHouse,
    TensorBase,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ENV {
    Development,
    Testing,
    Production,
}

impl fmt::Display for ENV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ENV::Development => write!(f, "Development"),
            ENV::Testing => write!(f, "Testing"),
            ENV::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for ENV {
    fn from(env: &str) -> Self {
        match env {
            "Testing" => ENV::Testing,
            "Production" => ENV::Production,
            _ => ENV::Development,
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "Development".into());
        let mut s = Config::new();
        s.set("env", env.clone())?;

        s.merge(File::with_name(CONFIG_FILE_PATH))?;

        // This makes it so "EA_SERVER__PORT overrides server.port
        s.merge(Environment::with_prefix("ariadne").separator("_"))?;

        s.try_into()
    }
}
