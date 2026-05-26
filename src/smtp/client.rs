use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub struct SmtpClient {
    transport: SmtpTransport,
}

impl SmtpClient {
    pub fn new(host: &str, port: u16, username: &str, password: &str) -> Self {
        let creds = Credentials::new(username.to_string(), password.to_string());
        let transport = SmtpTransport::relay(host)
            .unwrap()
            .port(port)
            .credentials(creds)
            .build();

        Self { transport }
    }

    pub async fn send(&self, message: Message) -> anyhow::Result<()> {
        self.transport.send(&message)?;
        Ok(())
    }
}
