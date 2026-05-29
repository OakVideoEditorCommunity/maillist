use hickory_resolver::{TokioResolver, config::ResolverConfig};
use std::collections::HashMap;

pub struct DnsCheckService;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DnsVerificationResult {
    pub spf: SpfCheckResult,
    pub dkim: DkimCheckResult,
    pub dmarc: DmarcCheckResult,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpfCheckResult {
    pub found: bool,
    pub valid: bool,
    pub record: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DkimCheckResult {
    pub found: bool,
    pub valid: bool,
    pub record: Option<String>,
    pub message: String,
    pub parsed: Option<DkimParsedRecord>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DkimParsedRecord {
    pub version: String,
    pub key_type: String,
    pub public_key: String,
    pub service_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DmarcCheckResult {
    pub found: bool,
    pub valid: bool,
    pub record: Option<String>,
    pub message: String,
    pub parsed: Option<DmarcParsedRecord>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DmarcParsedRecord {
    pub version: String,
    pub policy: String,
    pub subdomain_policy: Option<String>,
    pub percentage: Option<u8>,
    pub report_uris_aggregate: Vec<String>,
    pub report_uris_forensic: Vec<String>,
    pub dkim_alignment: Option<String>,
    pub spf_alignment: Option<String>,
    pub failure_options: Option<String>,
    pub report_interval: Option<u64>,
    pub report_format: Option<String>,
}

impl DnsCheckService {
    fn create_resolver() -> anyhow::Result<TokioResolver> {
        let resolver = TokioResolver::builder_with_config(
            ResolverConfig::default(),
            hickory_resolver::name_server::TokioConnectionProvider::default(),
        )
        .build();
        Ok(resolver)
    }

    pub async fn verify_spf(domain: &str) -> anyhow::Result<SpfCheckResult> {
        let resolver = Self::create_resolver()?;
        let lookup = resolver.txt_lookup(domain).await;

        match lookup {
            Ok(records) => {
                for record in records.iter() {
                    let txt = record.to_string();
                    if txt.contains("v=spf1") {
                        let valid = txt.contains("ip4:")
                            || txt.contains("include:")
                            || txt.contains("a:")
                            || txt.contains("mx:");
                        return Ok(SpfCheckResult {
                            found: true,
                            valid,
                            record: Some(txt),
                            message: if valid {
                                "SPF record found and appears valid".to_string()
                            } else {
                                "SPF record found but may not allow this server".to_string()
                            },
                        });
                    }
                }
                Ok(SpfCheckResult {
                    found: false,
                    valid: false,
                    record: None,
                    message: "No SPF record found".to_string(),
                })
            }
            Err(e) => Ok(SpfCheckResult {
                found: false,
                valid: false,
                record: None,
                message: format!("DNS lookup failed: {}", e),
            }),
        }
    }

    pub async fn verify_dkim(
        domain: &str,
        selector: &str,
        expected_public_key: Option<&str>,
    ) -> anyhow::Result<DkimCheckResult> {
        let resolver = Self::create_resolver()?;
        let lookup_name = format!("{}._domainkey.{}", selector, domain);
        let lookup = resolver.txt_lookup(&lookup_name).await;

        match lookup {
            Ok(records) => {
                for record in records.iter() {
                    let txt = record.to_string();
                    if txt.contains("v=DKIM1") {
                        let has_key = txt.contains("p=");
                        let key_matches = expected_public_key
                            .is_none_or(|expected| txt.contains(&format!("p={}", expected)));
                        let valid = has_key && key_matches;
                        let parsed = Self::parse_dkim_record(&txt);
                        return Ok(DkimCheckResult {
                            found: true,
                            valid,
                            record: Some(txt),
                            message: if !has_key {
                                "DKIM record found but public key is missing (revoked?)".to_string()
                            } else if !key_matches {
                                "DKIM record found but public key does not match".to_string()
                            } else {
                                "DKIM record found and public key matches".to_string()
                            },
                            parsed,
                        });
                    }
                }
                Ok(DkimCheckResult {
                    found: false,
                    valid: false,
                    record: None,
                    message: "No DKIM record found".to_string(),
                    parsed: None,
                })
            }
            Err(e) => Ok(DkimCheckResult {
                found: false,
                valid: false,
                record: None,
                message: format!("DNS lookup failed: {}", e),
                parsed: None,
            }),
        }
    }

    pub async fn verify_dmarc(domain: &str) -> anyhow::Result<DmarcCheckResult> {
        let resolver = Self::create_resolver()?;
        let lookup_name = format!("_dmarc.{}", domain);
        let lookup = resolver.txt_lookup(&lookup_name).await;

        match lookup {
            Ok(records) => {
                for record in records.iter() {
                    let txt = record.to_string();
                    if txt.contains("v=DMARC1") {
                        let parsed = Self::parse_dmarc_record(&txt);
                        let has_policy = parsed.is_some()
                            && ["none", "quarantine", "reject"]
                                .contains(&parsed.as_ref().unwrap().policy.as_str());
                        return Ok(DmarcCheckResult {
                            found: true,
                            valid: has_policy,
                            record: Some(txt),
                            message: if has_policy {
                                "DMARC record found and has a valid policy".to_string()
                            } else {
                                "DMARC record found but policy is missing or invalid".to_string()
                            },
                            parsed,
                        });
                    }
                }
                Ok(DmarcCheckResult {
                    found: false,
                    valid: false,
                    record: None,
                    message: "No DMARC record found".to_string(),
                    parsed: None,
                })
            }
            Err(e) => Ok(DmarcCheckResult {
                found: false,
                valid: false,
                record: None,
                message: format!("DNS lookup failed: {}", e),
                parsed: None,
            }),
        }
    }

    fn parse_dmarc_record(record: &str) -> Option<DmarcParsedRecord> {
        let mut tags: HashMap<String, String> = HashMap::new();
        for part in record.split(';') {
            let part = part.trim();
            if let Some(eq) = part.find('=') {
                let key = part[..eq].trim().to_lowercase();
                let value = part[eq + 1..].trim().to_string();
                tags.insert(key, value);
            }
        }

        let version = tags.get("v")?.clone();
        let policy = tags.get("p")?.clone();

        let parse_uris = |s: &str| {
            s.split(',')
                .map(|u| u.trim().to_string())
                .filter(|u| !u.is_empty())
                .collect::<Vec<_>>()
        };

        Some(DmarcParsedRecord {
            version,
            policy,
            subdomain_policy: tags.get("sp").cloned(),
            percentage: tags.get("pct").and_then(|v| v.parse().ok()),
            report_uris_aggregate: tags.get("rua").map(|s| parse_uris(s)).unwrap_or_default(),
            report_uris_forensic: tags.get("ruf").map(|s| parse_uris(s)).unwrap_or_default(),
            dkim_alignment: tags.get("adkim").cloned(),
            spf_alignment: tags.get("aspf").cloned(),
            failure_options: tags.get("fo").cloned(),
            report_interval: tags.get("ri").and_then(|v| v.parse().ok()),
            report_format: tags.get("rf").cloned(),
        })
    }

    fn parse_dkim_record(record: &str) -> Option<DkimParsedRecord> {
        let mut tags: HashMap<String, String> = HashMap::new();
        for part in record.split(';') {
            let part = part.trim();
            if let Some(eq) = part.find('=') {
                let key = part[..eq].trim().to_lowercase();
                let value = part[eq + 1..].trim().to_string();
                tags.insert(key, value);
            }
        }

        let version = tags.get("v")?.clone();
        let key_type = tags.get("k")?.clone();
        let public_key = tags.get("p")?.clone();

        Some(DkimParsedRecord {
            version,
            key_type,
            public_key,
            service_type: tags.get("t").cloned(),
            notes: tags.get("n").cloned(),
        })
    }

    pub async fn verify_all(
        domain: &str,
        selector: Option<&str>,
        expected_public_key: Option<&str>,
    ) -> anyhow::Result<DnsVerificationResult> {
        let spf = Self::verify_spf(domain).await?;
        let dkim = if let Some(sel) = selector {
            Self::verify_dkim(domain, sel, expected_public_key).await?
        } else {
            DkimCheckResult {
                found: false,
                valid: false,
                record: None,
                message: "No DKIM selector configured".to_string(),
                parsed: None,
            }
        };
        let dmarc = Self::verify_dmarc(domain).await?;

        Ok(DnsVerificationResult { spf, dkim, dmarc })
    }
}
