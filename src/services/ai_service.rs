use crate::ai::client::AiClient;
use crate::config::AiModerationConfig;

pub struct AiService {
    client: AiClient,
    config: AiModerationConfig,
}

#[derive(Debug, Clone)]
pub struct AiModerationResult {
    pub overall_score: i32,
    pub verdict: String,
    pub categories: serde_json::Value,
    pub flagged_categories: Vec<String>,
    pub llm_raw_output: Option<String>,
    pub risk_level: String,
}

impl AiService {
    pub fn new(config: AiModerationConfig) -> Self {
        let client = AiClient::new(config.clone());
        Self { client, config }
    }

    pub async fn moderate_email(
        &self,
        subject: &str,
        body: &str,
    ) -> anyhow::Result<AiModerationResult> {
        let result = self.client.analyze(subject, body).await?;
        Ok(result)
    }

    pub fn should_flag(&self, score: i32) -> bool {
        score >= self.config.high_risk_threshold
    }

    pub fn should_caution(&self, score: i32) -> bool {
        score >= self.config.medium_risk_threshold && score < self.config.high_risk_threshold
    }
}
