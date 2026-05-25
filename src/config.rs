use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
    pub refresh_token_expiration_days: i64,
    pub session_token_expiration_seconds: i64,
    pub password_min_length: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpIncomingConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpOutgoingConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub incoming: SmtpIncomingConfig,
    pub outgoing: SmtpOutgoingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiModerationConfig {
    pub enabled: bool,
    pub provider: String,
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub high_risk_threshold: i32,
    pub medium_risk_threshold: i32,
    pub request_timeout_seconds: u64,
    pub max_text_length: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveConfig {
    pub enabled: bool,
    pub storage_path: String,
    pub max_attachment_size_mb: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub smtp: SmtpConfig,
    pub ai_moderation: AiModerationConfig,
    pub archive: ArchiveConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

        let mut builder = Config::builder()
            .add_source(File::from(Path::new(&config_dir).join("default.toml")).required(true));

        let env_file = Path::new(&config_dir).join(format!("{}.toml", run_mode));
        if env_file.exists() {
            builder = builder.add_source(File::from(env_file).required(false));
        }

        builder = builder.add_source(
            Environment::with_prefix("OAK")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder.build()?;
        config.try_deserialize()
    }
}
