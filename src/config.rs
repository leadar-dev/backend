use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct BrokerConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DragonflyConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub bot_token: String,
    pub allowed_ids: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8000
}

#[derive(Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub broker: BrokerConfig,
    pub dragonfly: DragonflyConfig,
    pub auth: AuthConfig,
    pub server: ServerConfig,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database = envy::prefixed("DATABASE__")
            .from_env::<DatabaseConfig>()
            .context("DATABASE__ config invalid")?;
        let broker = envy::prefixed("BROKER__")
            .from_env::<BrokerConfig>()
            .context("BROKER__ config invalid")?;
        let dragonfly = envy::prefixed("DRAGONFLY__")
            .from_env::<DragonflyConfig>()
            .context("DRAGONFLY__ config invalid")?;
        let auth = envy::prefixed("AUTH__")
            .from_env::<AuthConfig>()
            .context("AUTH__ config invalid")?;
        let server = envy::prefixed("SERVER__")
            .from_env::<ServerConfig>()
            .unwrap_or(ServerConfig { port: 8000 });
        Ok(Config {
            database,
            broker,
            dragonfly,
            auth,
            server,
        })
    }

    pub fn allowed_telegram_ids(&self) -> Vec<i64> {
        self.auth
            .allowed_ids
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect()
    }
}
