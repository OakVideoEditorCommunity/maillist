use crate::ai::aliyun_signer::AliyunV3Signer;
use crate::config::AiModerationConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct AiClient {
    http: Client,
    signer: AliyunV3Signer,
    endpoint: String,
    service: String,
    config: AiModerationConfig,
}

#[derive(Debug, Serialize)]
struct AliyunRequest {
    #[serde(rename = "Service")]
    service: String,
    #[serde(rename = "ServiceParameters")]
    service_parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AliyunResponse {
    #[serde(rename = "Code")]
    code: i32,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "Data")]
    data: Option<AliyunData>,
}

#[derive(Debug, Deserialize)]
struct AliyunData {
    #[serde(rename = "RiskLevel")]
    risk_level: String,
    #[serde(rename = "Result")]
    result: Vec<AliyunResultItem>,
    #[serde(rename = "DataId")]
    data_id: Option<String>,
    #[serde(rename = "AccountId")]
    account_id: Option<String>,
    #[serde(rename = "TranslatedContent")]
    translated_content: Option<String>,
    #[serde(rename = "Ext")]
    ext: Option<AliyunExt>,
}

#[derive(Debug, Deserialize)]
struct AliyunExt {
    #[serde(rename = "LlmContent")]
    llm_content: Option<AliyunLlmContent>,
}

#[derive(Debug, Deserialize)]
struct AliyunLlmContent {
    #[serde(rename = "OutputText")]
    output_text: String,
}

#[derive(Debug, Deserialize, Clone)]
struct AliyunResultItem {
    #[serde(rename = "Label")]
    label: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Confidence")]
    confidence: Option<f64>,
    #[serde(rename = "RiskWords")]
    risk_words: Option<String>,
}

impl AiClient {
    pub fn new(config: AiModerationConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        let signer = AliyunV3Signer::new(
            config.access_key_id.clone(),
            config.access_key_secret.clone(),
            config.region.clone(),
            "green".to_string(),
        );

        Self {
            http,
            signer,
            endpoint: config.endpoint.clone(),
            service: config.service.clone(),
            config,
        }
    }

    pub async fn analyze(
        &self,
        subject: &str,
        body: &str,
    ) -> anyhow::Result<crate::services::ai_service::AiModerationResult> {
        if !self.config.enabled {
            return Ok(crate::services::ai_service::AiModerationResult {
                overall_score: 0,
                verdict: "clean".to_string(),
                categories: json!({}),
                flagged_categories: vec![],
                llm_raw_output: None,
                risk_level: "none".to_string(),
            });
        }

        let text = format!("{} {}", subject, body);
        let truncated = if text.len() > self.config.max_text_length {
            &text[..self.config.max_text_length]
        } else {
            &text
        };

        let request_body = AliyunRequest {
            service: self.service.clone(),
            service_parameters: json!({
                "content": truncated,
                "dataId": format!("email_{}", uuid::Uuid::new_v4()),
            }),
        };

        let body_json = serde_json::to_string(&request_body)?;
        let body_bytes = body_json.as_bytes();

        let extra_headers = self.signer.sign_request(
            "POST",
            "/",
            "Action=TextModerationPlus",
            &[
                ("host".to_string(), self.endpoint.replace("https://", "")),
                ("content-type".to_string(), "application/json; charset=utf-8".to_string()),
            ],
            body_bytes,
        );

        let mut request = self
            .http
            .post(format!("{}/?Action=TextModerationPlus", self.endpoint))
            .header("Content-Type", "application/json; charset=utf-8")
            .body(body_json);

        for (key, value) in extra_headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Aliyun API error: HTTP {} - {}",
                status,
                response_text
            ));
        }

        let aliyun_resp: AliyunResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse Aliyun response: {}. Raw: {}", e, response_text))?;

        if aliyun_resp.code != 200 {
            return Err(anyhow::anyhow!(
                "Aliyun API error: {} - {}",
                aliyun_resp.code,
                aliyun_resp.message
            ));
        }

        let data = aliyun_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in Aliyun response"))?;

        let mut max_confidence = 0.0;
        let mut flagged = Vec::new();
        let mut categories = serde_json::Map::new();

        for item in &data.result {
            let confidence = item.confidence.unwrap_or(0.0);
            if confidence > max_confidence {
                max_confidence = confidence;
            }

            let category_obj = json!({
                "score": confidence,
                "description": item.description,
                "risk_words": item.risk_words,
            });
            categories.insert(item.label.clone(), category_obj);

            if confidence >= self.config.high_risk_threshold as f64 {
                flagged.push(item.label.clone());
            }
        }

        let verdict = match data.risk_level.as_str() {
            "high" => "flagged",
            "medium" => "caution",
            _ => "clean",
        };

        let llm_raw_output = data
            .ext
            .as_ref()
            .and_then(|e| e.llm_content.as_ref())
            .map(|l| l.output_text.clone());

        Ok(crate::services::ai_service::AiModerationResult {
            overall_score: max_confidence as i32,
            verdict: verdict.to_string(),
            categories: serde_json::Value::Object(categories),
            flagged_categories: flagged,
            llm_raw_output,
            risk_level: data.risk_level,
        })
    }
}
