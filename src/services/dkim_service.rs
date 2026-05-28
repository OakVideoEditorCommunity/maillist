use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::EncodeRsaPublicKey,
    pkcs1v15::SigningKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey},
    rand_core::OsRng,
    signature::{SignatureEncoding, Signer},
};
use sha2::{Digest, Sha256};

const DKIM_SIGNATURE_HEADER: &str = "DKIM-Signature";

pub struct DkimService;

#[derive(Debug, Clone)]
pub struct DkimKeypair {
    pub private_key_pem: String,
    pub public_key_base64: String,
    pub selector: String,
}

#[derive(Debug, Clone)]
pub struct DkimSignatureResult {
    pub header_name: String,
    pub header_value: String,
}

impl DkimService {
    pub fn generate_keypair(domain: &str) -> anyhow::Result<DkimKeypair> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);

        let private_key_pem = private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;

        let public_key_der = public_key.to_pkcs1_der()?;
        let public_key_base64 = BASE64.encode(public_key_der.as_bytes());

        let selector = format!("oak-{}", &domain.replace('.', "-"));

        Ok(DkimKeypair {
            private_key_pem: private_key_pem.to_string(),
            public_key_base64,
            selector,
        })
    }

    pub fn sign_message(
        private_key_pem: &str,
        domain: &str,
        selector: &str,
        headers: &[(String, String)],
        body: &str,
    ) -> anyhow::Result<DkimSignatureResult> {
        let private_key = Self::load_private_key(private_key_pem)?;
        let signing_key = SigningKey::<Sha256>::new(private_key);

        let canonicalized_body = Self::canonicalize_body_relaxed(body);
        let body_hash = BASE64.encode(Sha256::digest(&canonicalized_body));

        let headers_to_sign = ["from", "to", "subject", "date"];
        let signed_headers: Vec<(&str, &str)> = headers_to_sign
            .iter()
            .filter_map(|name| {
                headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map(|(_, v)| (*name, v.as_str()))
            })
            .collect();

        let h_tag = signed_headers
            .iter()
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
            .join(":");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let mut dkim_header = format!(
            "v=1; a=rsa-sha256; c=relaxed/relaxed; d={domain}; s={selector}; t={timestamp}; bh={body_hash}; h={h_tag}; b=",
        );

        let canonicalized_headers = Self::canonicalize_headers_relaxed(&signed_headers);
        let dkim_header_canon =
            Self::canonicalize_header_relaxed(DKIM_SIGNATURE_HEADER, &dkim_header);

        let sign_input = format!("{}{}\r\n", canonicalized_headers, dkim_header_canon);
        let signature = signing_key.sign(sign_input.as_bytes());
        let signature_b64 = BASE64.encode(signature.to_bytes());

        dkim_header.push_str(&signature_b64);

        Ok(DkimSignatureResult {
            header_name: DKIM_SIGNATURE_HEADER.to_string(),
            header_value: dkim_header,
        })
    }

    pub fn build_dns_record(public_key_base64: &str) -> String {
        format!("v=DKIM1; k=rsa; p={public_key_base64}")
    }

    pub fn build_spf_record(server_ip: &str) -> String {
        format!("v=spf1 ip4:{server_ip} ~all")
    }

    pub fn build_dmarc_record(domain: &str) -> String {
        format!("v=DMARC1; p=quarantine; rua=mailto:dmarc@{domain}")
    }

    fn load_private_key(pem: &str) -> anyhow::Result<RsaPrivateKey> {
        let key = RsaPrivateKey::from_pkcs8_pem(pem)?;
        Ok(key)
    }

    fn canonicalize_body_relaxed(body: &str) -> Vec<u8> {
        let lines: Vec<&str> = body.lines().collect();
        let mut result = Vec::new();
        let mut empty_count = 0;

        for line in &lines {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                empty_count += 1;
            } else {
                for _ in 0..empty_count {
                    result.extend_from_slice(b"\r\n");
                }
                empty_count = 0;
                result.extend_from_slice(trimmed.as_bytes());
                result.extend_from_slice(b"\r\n");
            }
        }

        if result.is_empty() {
            result.extend_from_slice(b"\r\n");
        }

        result
    }

    fn canonicalize_header_relaxed(name: &str, value: &str) -> String {
        let name = name.to_lowercase();
        let value = value
            .lines()
            .map(|l| {
                let t = l.trim();
                t.split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!("{}:{}\r\n", name, value)
    }

    fn canonicalize_headers_relaxed(headers: &[(&str, &str)]) -> String {
        headers
            .iter()
            .map(|(k, v)| Self::canonicalize_header_relaxed(k, v))
            .collect::<String>()
    }
}
