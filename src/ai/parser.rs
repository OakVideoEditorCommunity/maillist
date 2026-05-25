use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AiResponse {
    pub overall_score: i32,
    pub verdict: String,
    pub categories: serde_json::Value,
    pub flagged_categories: Vec<String>,
}

pub fn parse_ai_response(json: &str) -> anyhow::Result<AiResponse> {
    let response: AiResponse = serde_json::from_str(json)?;
    Ok(response)
}
