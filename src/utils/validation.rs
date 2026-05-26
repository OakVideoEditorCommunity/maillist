use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
}

pub fn is_valid_email(email: &str) -> bool {
    if email.len() > 254 {
        return false;
    }
    EMAIL_REGEX.is_match(email)
}

pub fn is_valid_domain(domain: &str) -> bool {
    let domain_regex = Regex::new(
        r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$"
    ).unwrap();
    domain_regex.is_match(domain) && domain.len() <= 253
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
