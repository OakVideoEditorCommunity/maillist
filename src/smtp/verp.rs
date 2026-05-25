pub struct VerpAddress;

impl VerpAddress {
    pub fn encode(local_part: &str, domain: &str, token: &str) -> String {
        format!("{}-bounces+{}@{}", local_part, token, domain)
    }

    pub fn decode(address: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = address.split('@').collect();
        if parts.len() != 2 {
            return None;
        }
        let local = parts[0];
        let domain = parts[1];

        if let Some(pos) = local.find("-bounces+") {
            let list_local = &local[..pos];
            let token = &local[pos + 9..];
            return Some((list_local.to_string(), domain.to_string(), token.to_string()));
        }

        None
    }
}
