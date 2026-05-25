pub fn extract_local_part(email: &str) -> Option<&str> {
    email.split('@').next()
}

pub fn extract_domain(email: &str) -> Option<&str> {
    email.split('@').nth(1)
}

pub fn build_list_email(local_part: &str, domain: &str) -> String {
    format!("{}@{}", local_part, domain)
}

pub fn build_verp_address(local_part: &str, domain: &str, token: &str) -> String {
    format!("{}-bounces+{}@{}", local_part, token, domain)
}
