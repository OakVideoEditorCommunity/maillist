use crate::config::AiModerationConfig;
use reqwest::Client;

pub struct AiClient {
    http: Client,
    config: AiModerationConfig,
}

impl AiClient {
    pub fn new(config: AiModerationConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        Self { http, config }
    }

    pub async fn analyze(&self, _text: &str) -> anyhow::Result<serde_json::Value> {
        todo!()
    }
}
