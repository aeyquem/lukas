use std::{net::SocketAddr, time::Duration};

use thiserror::Error;
use url::Url;

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";
const DEFAULT_MAX_CONNECTIONS: &str = "5";
const DEFAULT_CONNECT_TIMEOUT_SECONDS: &str = "5";
const DEFAULT_ACQUIRE_TIMEOUT_SECONDS: &str = "5";
const DEFAULT_DATABASE_STARTUP_TIMEOUT_SECONDS: &str = "30";
const DEFAULT_SHUTDOWN_TIMEOUT_SECONDS: &str = "10";

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub canonical_origin: String,
    pub bind_address: SocketAddr,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub acquire_timeout: Duration,
    pub database_startup_timeout: Duration,
    pub shutdown_timeout: Duration,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required setting {0}")]
    Missing(&'static str),
    #[error("invalid {0}: {1}")]
    Invalid(&'static str, &'static str),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    fn from_lookup<F>(mut get: F) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let database_url = required(&mut get, "DATABASE_URL")?;
        validate_database_url(&database_url)?;

        let canonical_origin = required(&mut get, "CANONICAL_ORIGIN")?;
        validate_canonical_origin(&canonical_origin)?;

        let bind_address = optional(&mut get, "API_BIND_ADDR", DEFAULT_BIND_ADDRESS)?
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::Invalid("API_BIND_ADDR", "must be an IP address and port"))?;

        let max_connections = bounded_u32(
            &mut get,
            "DATABASE_MAX_CONNECTIONS",
            DEFAULT_MAX_CONNECTIONS,
            1,
            100,
        )?;
        let connect_timeout = bounded_seconds(
            &mut get,
            "DATABASE_CONNECT_TIMEOUT",
            DEFAULT_CONNECT_TIMEOUT_SECONDS,
            1,
            300,
        )?;
        let acquire_timeout = bounded_seconds(
            &mut get,
            "DATABASE_ACQUIRE_TIMEOUT",
            DEFAULT_ACQUIRE_TIMEOUT_SECONDS,
            1,
            300,
        )?;
        let database_startup_timeout = bounded_seconds(
            &mut get,
            "DATABASE_STARTUP_TIMEOUT",
            DEFAULT_DATABASE_STARTUP_TIMEOUT_SECONDS,
            1,
            600,
        )?;
        let shutdown_timeout = bounded_seconds(
            &mut get,
            "SHUTDOWN_TIMEOUT",
            DEFAULT_SHUTDOWN_TIMEOUT_SECONDS,
            1,
            600,
        )?;

        Ok(Self {
            database_url,
            canonical_origin,
            bind_address,
            max_connections,
            connect_timeout,
            acquire_timeout,
            database_startup_timeout,
            shutdown_timeout,
        })
    }
}

fn required<F>(get: &mut F, name: &'static str) -> Result<String, ConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    get(name)
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::Missing(name))
}

fn optional<F>(get: &mut F, name: &'static str, default: &str) -> Result<String, ConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    match get(name) {
        Some(value) if !value.trim().is_empty() => Ok(value),
        Some(_) => Err(ConfigError::Invalid(name, "must not be empty")),
        None => Ok(default.to_owned()),
    }
}

fn bounded_u32<F>(
    get: &mut F,
    name: &'static str,
    default: &str,
    minimum: u32,
    maximum: u32,
) -> Result<u32, ConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    let value = optional(get, name, default)?
        .parse::<u32>()
        .map_err(|_| ConfigError::Invalid(name, "must be a whole number"))?;

    if !(minimum..=maximum).contains(&value) {
        return Err(ConfigError::Invalid(name, "is outside the accepted range"));
    }

    Ok(value)
}

fn bounded_seconds<F>(
    get: &mut F,
    name: &'static str,
    default: &str,
    minimum: u64,
    maximum: u64,
) -> Result<Duration, ConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    bounded_u64(get, name, default, minimum, maximum).map(Duration::from_secs)
}

fn bounded_u64<F>(
    get: &mut F,
    name: &'static str,
    default: &str,
    minimum: u64,
    maximum: u64,
) -> Result<u64, ConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    let value = optional(get, name, default)?
        .parse::<u64>()
        .map_err(|_| ConfigError::Invalid(name, "must be a whole number"))?;

    if !(minimum..=maximum).contains(&value) {
        return Err(ConfigError::Invalid(name, "is outside the accepted range"));
    }

    Ok(value)
}

fn validate_database_url(value: &str) -> Result<(), ConfigError> {
    let url = Url::parse(value)
        .map_err(|_| ConfigError::Invalid("DATABASE_URL", "must be a PostgreSQL connection URL"))?;

    if !matches!(url.scheme(), "postgres" | "postgresql") || url.host_str().is_none() {
        return Err(ConfigError::Invalid(
            "DATABASE_URL",
            "must be a PostgreSQL connection URL",
        ));
    }

    Ok(())
}

fn validate_canonical_origin(value: &str) -> Result<(), ConfigError> {
    let url = Url::parse(value)
        .map_err(|_| ConfigError::Invalid("CANONICAL_ORIGIN", "must be an HTTP origin"))?;

    let is_http = matches!(url.scheme(), "http" | "https");
    let has_credentials = !url.username().is_empty() || url.password().is_some();
    let has_path_or_query = url.path() != "/" || url.query().is_some() || url.fragment().is_some();
    let exact_origin = url.origin().ascii_serialization() == value;

    if !is_http || url.host_str().is_none() || has_credentials || has_path_or_query || !exact_origin
    {
        return Err(ConfigError::Invalid(
            "CANONICAL_ORIGIN",
            "must be an HTTP(S) origin without credentials, path, query, or fragment",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn configured(values: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let values: HashMap<String, String> = values
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        Config::from_lookup(|name| values.get(name).cloned())
    }

    fn required_settings() -> [(&'static str, &'static str); 2] {
        [
            (
                "DATABASE_URL",
                "postgres://lukas:secret@postgres:5432/lukas",
            ),
            ("CANONICAL_ORIGIN", "http://localhost:4173"),
        ]
    }

    #[test]
    fn required_settings_are_required_without_echoing_values() {
        let error = configured(&[("CANONICAL_ORIGIN", "http://localhost:4173")]).unwrap_err();
        assert_eq!(error, ConfigError::Missing("DATABASE_URL"));
        assert!(!error.to_string().contains("secret"));

        let error = configured(&[("DATABASE_URL", "postgres://secret")]).unwrap_err();
        assert_eq!(error, ConfigError::Missing("CANONICAL_ORIGIN"));
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn documented_defaults_are_used() {
        let config = configured(&required_settings()).unwrap();

        assert_eq!(config.bind_address, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.acquire_timeout, Duration::from_secs(5));
        assert_eq!(config.database_startup_timeout, Duration::from_secs(30));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(10));
    }

    #[test]
    fn valid_overrides_are_parsed() {
        let mut settings = required_settings().to_vec();
        settings.extend([
            ("API_BIND_ADDR", "127.0.0.1:9000"),
            ("DATABASE_MAX_CONNECTIONS", "8"),
            ("DATABASE_CONNECT_TIMEOUT", "20"),
            ("DATABASE_ACQUIRE_TIMEOUT", "25"),
            ("DATABASE_STARTUP_TIMEOUT", "45"),
            ("SHUTDOWN_TIMEOUT", "30"),
        ]);
        let config = configured(&settings).unwrap();

        assert_eq!(config.bind_address, "127.0.0.1:9000".parse().unwrap());
        assert_eq!(config.max_connections, 8);
        assert_eq!(config.connect_timeout, Duration::from_secs(20));
        assert_eq!(config.acquire_timeout, Duration::from_secs(25));
        assert_eq!(config.database_startup_timeout, Duration::from_secs(45));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn invalid_origin_and_out_of_range_values_are_rejected() {
        let settings = required_settings();
        for origin in [
            "file://localhost:4173",
            "http://user:pass@localhost:4173",
            "http://localhost:4173/path",
            "http://localhost:4173/",
            "http://localhost:4173?query=1",
        ] {
            let values = [settings[0], ("CANONICAL_ORIGIN", origin)];
            assert!(
                configured(&values).is_err(),
                "accepted invalid origin {origin}"
            );
        }

        for (name, value) in [
            ("DATABASE_MAX_CONNECTIONS", "0"),
            ("DATABASE_MAX_CONNECTIONS", "101"),
            ("DATABASE_CONNECT_TIMEOUT", "0"),
            ("DATABASE_CONNECT_TIMEOUT", "301"),
            ("DATABASE_ACQUIRE_TIMEOUT", "NaN"),
            ("DATABASE_STARTUP_TIMEOUT", "601"),
            ("SHUTDOWN_TIMEOUT", "601"),
        ] {
            let mut values = settings.to_vec();
            values.push((name, value));
            assert!(configured(&values).is_err(), "accepted {name}={value}");
        }
    }

    #[test]
    fn database_url_errors_do_not_include_credentials() {
        let error = configured(&[
            ("DATABASE_URL", "postgres://user:very-secret@"),
            ("CANONICAL_ORIGIN", "http://localhost:4173"),
        ])
        .unwrap_err();

        assert!(matches!(error, ConfigError::Invalid("DATABASE_URL", _)));
        assert!(!error.to_string().contains("very-secret"));
    }
}
