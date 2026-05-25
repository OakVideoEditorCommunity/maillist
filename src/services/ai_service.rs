use crate::config::AiModerationConfig;

pub struct AiService {
    config: AiModerationConfig,
}

impl AiService {
    pub fn new(config: AiModerationConfig) -> Self {
        Self { config }
    }

    pub async fn moderate_content(&self, _subject: &str, _body: &str) -> anyhow::Result<AiModerationResult> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct AiModerationResult {
    pub overall_score: i32,
    pub verdict: String,
    pub categories: serde_json::Value,
    pub flagged_categories: Vec<String>,
}
