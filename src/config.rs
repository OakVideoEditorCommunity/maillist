use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
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
    pub webauthn_rp_id: Option<String>,
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
    pub access_key_id: String,
    pub access_key_secret: String,
    pub region: String,
    pub service: String,
    pub endpoint: String,
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
pub struct BrandingConfig {
    pub site_name: String,
    pub primary_color: String,
    pub logo_url: String,
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
    pub branding: BrandingConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());
        let default_path = Path::new(&config_dir).join("default.toml");

        if !default_path.exists() {
            Self::write_default_config(&config_dir)?;
        }

        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

        let mut builder =
            Config::builder().add_source(File::from(default_path.clone()).required(true));

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

    pub fn default_config_path() -> std::path::PathBuf {
        let config_dir = std::env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());
        Path::new(&config_dir).join("default.toml")
    }

    fn write_default_config(config_dir: &str) -> Result<(), ConfigError> {
        let dir = Path::new(config_dir);
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(|e| {
                ConfigError::Message(format!("Failed to create config directory: {}", e))
            })?;
        }

        let default_toml = r##"[server]
host = "0.0.0.0"
port = 3000
base_url = "http://localhost:3000"

[database]
url = "sqlite://./data.db?mode=rwc"
max_connections = 10
min_connections = 2
connect_timeout = 10
idle_timeout = 300

[security]
jwt_secret = "CHANGE_ME_IN_PRODUCTION"
jwt_expiration_seconds = 900
refresh_token_expiration_days = 7
session_token_expiration_seconds = 600
password_min_length = 8

[smtp.incoming]
enabled = true
host = "0.0.0.0"
port = 2525

[smtp.outgoing]
host = ""
port = 587
username = ""
password = ""
from_address = "noreply@example.com"

[ai_moderation]
enabled = false
provider = "aliyun"
access_key_id = ""
access_key_secret = ""
region = "cn-shanghai"
service = "ugc_moderation_byllm"
endpoint = "https://green-cip.cn-shanghai.aliyuncs.com"
high_risk_threshold = 80
medium_risk_threshold = 50
request_timeout_seconds = 30
max_text_length = 2000

[archive]
enabled = true
storage_path = "./storage/archives"
max_attachment_size_mb = 10

[logging]
level = "info"
format = "pretty"

[branding]
site_name = "Oak MailList"
primary_color = "#409EFF"
logo_url = ""
"##;

        let path = dir.join("default.toml");
        std::fs::write(&path, default_toml)
            .map_err(|e| ConfigError::Message(format!("Failed to write default config: {}", e)))
    }
}
