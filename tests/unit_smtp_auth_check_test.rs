use oak_maillist::smtp::auth_check::{AuthChecker, AuthCheckResult, DkimResult, DmarcResult, SpfResult};

#[tokio::test]
async fn test_auth_checker_check_returns_none() {
    let checker = AuthChecker::new();
    let result = checker.check("192.168.1.1", "example.com", "from@example.com").await;
    assert!(matches!(result.spf_result, SpfResult::None));
    assert!(matches!(result.dkim_result, DkimResult::None));
    assert!(matches!(result.dmarc_result, DmarcResult::None));
}

#[test]
fn test_is_suspicious_all_clean() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Pass,
        dkim_result: DkimResult::Pass,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(!checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_spf_fail() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Fail,
        dkim_result: DkimResult::Pass,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_spf_perm_error() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::PermError,
        dkim_result: DkimResult::Pass,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_dkim_fail() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Pass,
        dkim_result: DkimResult::Fail,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_dkim_perm_error() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Pass,
        dkim_result: DkimResult::PermError,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_dmarc_fail() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Pass,
        dkim_result: DkimResult::Pass,
        dmarc_result: DmarcResult::Fail,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_multiple_failures() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::Fail,
        dkim_result: DkimResult::Fail,
        dmarc_result: DmarcResult::Fail,
    };
    assert!(checker.is_suspicious(&result));
}

#[test]
fn test_is_suspicious_spf_softfail_not_suspicious() {
    let checker = AuthChecker::new();
    let result = AuthCheckResult {
        spf_result: SpfResult::SoftFail,
        dkim_result: DkimResult::Pass,
        dmarc_result: DmarcResult::Pass,
    };
    assert!(!checker.is_suspicious(&result));
}

#[tokio::test]
async fn test_auth_checker_default() {
    let checker: AuthChecker = Default::default();
    let result = checker.check("1.2.3.4", "test.com", "a@test.com").await;
    assert!(matches!(result.spf_result, SpfResult::None));
}
