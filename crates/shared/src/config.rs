use anyhow::Context;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, de};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    #[serde(default)]
    pub password_hasher: PasswordHasherSettings,
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

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordHasherSettings {
    /// Max concurrent Argon2 hashes. Each holds ~19 MiB and a full core.
    /// Unset means one per visible CPU.
    #[serde(default)]
    pub max_concurrent: Option<usize>,
    /// How long a request waits for a hashing slot before getting 503.
    #[serde(default = "default_hasher_acquire_timeout_ms")]
    pub acquire_timeout_ms: u64,
}

impl Default for PasswordHasherSettings {
    fn default() -> Self {
        Self {
            max_concurrent: None,
            acquire_timeout_ms: default_hasher_acquire_timeout_ms(),
        }
    }
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

fn default_hasher_acquire_timeout_ms() -> u64 {
    2000
}

fn default_environment() -> Environment {
    Environment::Production
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

        anyhow::ensure!(
            config.password_hasher.max_concurrent != Some(0),
            "PASSWORD_HASHER__MAX_CONCURRENT must be at least 1 (unset it to use one per CPU)"
        );

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn load(vars: &[(&str, &str)]) -> Config {
        let mut env: HashMap<String, String> = [
            ("DATABASE__USERNAME", "u"),
            ("DATABASE__PASSWORD", "p"),
            ("DATABASE__HOST", "h"),
            ("DATABASE__DATABASE_NAME", "d"),
        ]
        .into_iter()
        .chain(vars.iter().copied())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        env.insert("APPLICATION__PORT".to_string(), "8080".to_string());

        config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .source(Some(env)),
            )
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn password_hasher_settings_default_when_unset() {
        let config = load(&[]);
        assert_eq!(config.password_hasher.max_concurrent, None);
        assert_eq!(config.password_hasher.acquire_timeout_ms, 2000);
    }

    #[test]
    fn password_hasher_settings_read_from_env() {
        let config = load(&[
            ("PASSWORD_HASHER__MAX_CONCURRENT", "4"),
            ("PASSWORD_HASHER__ACQUIRE_TIMEOUT_MS", "500"),
        ]);
        assert_eq!(config.password_hasher.max_concurrent, Some(4));
        assert_eq!(config.password_hasher.acquire_timeout_ms, 500);
    }
}
