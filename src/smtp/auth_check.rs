use tracing::warn;

pub struct AuthCheckResult {
    pub spf_result: SpfResult,
    pub dkim_result: DkimResult,
    pub dmarc_result: DmarcResult,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpfResult {
    Pass,
    Fail,
    SoftFail,
    Neutral,
    None,
    TempError,
    PermError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DkimResult {
    Pass,
    Fail,
    Neutral,
    None,
    TempError,
    PermError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DmarcResult {
    Pass,
    Fail,
    None,
}

pub struct AuthChecker;

impl AuthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check(&self, _remote_ip: &str, _from_domain: &str, _envelope_from: &str) -> AuthCheckResult {
        // TODO: Implement actual SPF/DKIM/DMARC checks
        // This requires DNS queries and cryptographic verification
        // For now, return permissive results

        AuthCheckResult {
            spf_result: SpfResult::None,
            dkim_result: DkimResult::None,
            dmarc_result: DmarcResult::None,
        }
    }

    pub fn is_suspicious(&self, result: &AuthCheckResult) -> bool {
        matches!(result.spf_result, SpfResult::Fail | SpfResult::PermError)
            || matches!(result.dkim_result, DkimResult::Fail | DkimResult::PermError)
            || matches!(result.dmarc_result, DmarcResult::Fail)
    }
}

impl Default for AuthChecker {
    fn default() -> Self {
        Self::new()
    }
}
