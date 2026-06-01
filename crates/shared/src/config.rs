use anyhow::Context;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, de};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_environment")]
    pub environment: Environment,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub jwt_secret: Option<SecretString>,
    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    pub database_name: String,
    #[serde(default)]
    pub require_ssl: bool,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        let ssl = if self.require_ssl {
            "?sslmode=require"
        } else {
            ""
        };
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}/{}{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
            ssl,
        ))
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_db_port() -> u16 {
    5432
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

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a comma-separated string or a sequence of strings")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                out.push(s);
            }
            Ok(out)
        }
    }

    deserializer.deserialize_any(Visitor)
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
