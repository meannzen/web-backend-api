use anyhow::Context;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_environment")]
    pub environment: Environment,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: SecretString,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_port() -> u16 {
    3000
}

fn default_max_connections() -> u32 {
    10
}

fn default_environment() -> Environment {
    Environment::Development
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .context("failed to build configuration")?;

        let config: Config = config
            .try_deserialize()
            .context("failed to deserialize configuration")?;

        Ok(config)
    }
}
