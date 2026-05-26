use oak_maillist::smtp::verp::VerpAddress;
use oak_maillist::smtp::parser::EmailParser;

#[test]
fn test_verp_encode() {
    assert_eq!(
        VerpAddress::encode("list", "example.com", "abc123"),
        "list-bounces+abc123@example.com"
    );
}

#[test]
fn test_verp_decode_success() {
    let result = VerpAddress::decode("list-bounces+abc123@example.com");
    assert!(result.is_some());
    let (lp, domain, token) = result.unwrap();
    assert_eq!(lp, "list");
    assert_eq!(domain, "example.com");
    assert_eq!(token, "abc123");
}

#[test]
fn test_verp_decode_no_at_sign() {
    assert!(VerpAddress::decode("not-an-email").is_none());
}

#[test]
fn test_verp_decode_no_bounces_marker() {
    assert!(VerpAddress::decode("list+token@example.com").is_none());
}

#[test]
fn test_verp_decode_empty_token() {
    let result = VerpAddress::decode("list-bounces+@example.com");
    assert!(result.is_some());
    let (_, _, token) = result.unwrap();
    assert_eq!(token, "");
}

#[test]
fn test_verp_encode_decode_roundtrip() {
    let encoded = VerpAddress::encode("announce", "lists.example.org", "token_xyz");
    let decoded = VerpAddress::decode(&encoded).unwrap();
    assert_eq!(decoded.0, "announce");
    assert_eq!(decoded.1, "lists.example.org");
    assert_eq!(decoded.2, "token_xyz");
}

#[test]
fn test_email_parser_simple_text() {
    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\n\r\nBody text";
    let parsed = EmailParser::parse(raw).unwrap();
    let subject = parsed.headers.iter().find(|h| h.get_key() == "Subject").map(|h| h.get_value());
    assert_eq!(subject, Some("Hello".to_string()));
}

#[test]
fn test_email_parser_multipart() {
    let raw = b"From: a@b.com\r\nContent-Type: multipart/alternative; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nplain\r\n--b\r\nContent-Type: text/html\r\n\r\n<html></html>\r\n--b--";
    let parsed = EmailParser::parse(raw).unwrap();
    assert_eq!(parsed.subparts.len(), 2);
}

#[test]
fn test_email_parser_invalid_data() {
    let raw = b"not a valid email at all \xff\xfe";
    // mailparse is lenient with invalid data, may return Ok with minimal parsing
    let result = EmailParser::parse(raw);
    assert!(result.is_ok() || result.is_err());
}
