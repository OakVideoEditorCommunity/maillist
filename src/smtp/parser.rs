use mailparse::ParsedMail;

pub struct EmailParser;

impl EmailParser {
    pub fn parse(raw: &[u8]) -> anyhow::Result<ParsedMail<'_>> {
        let parsed = mailparse::parse_mail(raw)?;
        Ok(parsed)
    }
}
