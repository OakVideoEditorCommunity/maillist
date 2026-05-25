use crate::services::ai_service::AiModerationResult;

pub fn parse_ai_response(_content: &str) -> anyhow::Result<AiModerationResult> {
    // Aliyun API already returns structured JSON, parsing is done in client.rs
    // This function is kept for compatibility but not used for Aliyun
    Ok(AiModerationResult {
        overall_score: 0,
        verdict: "clean".to_string(),
        categories: serde_json::json!({}),
        flagged_categories: vec![],
        llm_raw_output: None,
        risk_level: "none".to_string(),
    })
}
