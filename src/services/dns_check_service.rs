use hickory_resolver::{TokioResolver, config::ResolverConfig};

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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DmarcCheckResult {
    pub found: bool,
    pub valid: bool,
    pub record: Option<String>,
    pub message: String,
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
                        });
                    }
                }
                Ok(DkimCheckResult {
                    found: false,
                    valid: false,
                    record: None,
                    message: "No DKIM record found".to_string(),
                })
            }
            Err(e) => Ok(DkimCheckResult {
                found: false,
                valid: false,
                record: None,
                message: format!("DNS lookup failed: {}", e),
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
                        let has_policy = txt.contains("p=none")
                            || txt.contains("p=quarantine")
                            || txt.contains("p=reject");
                        return Ok(DmarcCheckResult {
                            found: true,
                            valid: has_policy,
                            record: Some(txt),
                            message: if has_policy {
                                "DMARC record found and has a valid policy".to_string()
                            } else {
                                "DMARC record found but policy is missing or invalid".to_string()
                            },
                        });
                    }
                }
                Ok(DmarcCheckResult {
                    found: false,
                    valid: false,
                    record: None,
                    message: "No DMARC record found".to_string(),
                })
            }
            Err(e) => Ok(DmarcCheckResult {
                found: false,
                valid: false,
                record: None,
                message: format!("DNS lookup failed: {}", e),
            }),
        }
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
            }
        };
        let dmarc = Self::verify_dmarc(domain).await?;

        Ok(DnsVerificationResult { spf, dkim, dmarc })
    }
}
