use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

pub struct AliyunV3Signer {
    access_key_id: String,
    access_key_secret: String,
    region: String,
    product_code: String,
}

impl AliyunV3Signer {
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        region: String,
        product_code: String,
    ) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            region,
            product_code,
        }
    }

    pub fn sign_request(
        &self,
        method: &str,
        uri: &str,
        query: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Vec<(String, String)> {
        let mut result = Vec::new();

        let date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = uuid::Uuid::new_v4().to_string();
        let content_sha256 = sha256_hex(body);

        result.push(("x-acs-content-sha256".to_string(), content_sha256.clone()));
        result.push(("x-acs-date".to_string(), date.clone()));
        result.push(("x-acs-signature-nonce".to_string(), nonce.clone()));
        result.push(("x-acs-version".to_string(), "2022-03-02".to_string()));

        let mut all_headers = headers.to_vec();
        all_headers.extend(result.clone());
        all_headers.sort_by_key(|a| a.0.to_lowercase());

        let canonical_headers: Vec<String> = all_headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect();
        let signed_headers: Vec<String> = all_headers
            .iter()
            .map(|(k, _)| k.to_lowercase())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            uri,
            query,
            canonical_headers.join("\n"),
            signed_headers.join(";"),
            content_sha256
        );

        let canonical_request_hash = sha256_hex(canonical_request.as_bytes());
        let string_to_sign = format!("ACS3-HMAC-SHA256\n{}", canonical_request_hash);

        let signing_key = self.derive_signing_key();
        let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes());

        let authorization = format!(
            "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
            self.access_key_id,
            signed_headers.join(";"),
            signature
        );

        result.push(("Authorization".to_string(), authorization));
        result
    }

    fn derive_signing_key(&self) -> Vec<u8> {
        let secret = format!("aliyun_v3{}", self.access_key_secret);
        let k_date = hmac_sha256(secret.as_bytes(), b"acs");
        let k_region = hmac_sha256(&k_date, self.region.as_bytes());
        let k_product = hmac_sha256(&k_region, self.product_code.as_bytes());
        hmac_sha256(&k_product, b"aliyun_v4_request")
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let result = hmac_sha256(key, data);
    result.iter().map(|b| format!("{:02x}", b)).collect()
}
