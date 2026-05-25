use tokio::net::TcpListener;
use tracing::{info, error};

pub struct SmtpServer {
    host: String,
    port: u16,
}

impl SmtpServer {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("SMTP server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((_socket, peer)) => {
                    info!("SMTP connection from {}", peer);
                }
                Err(e) => {
                    error!("SMTP accept error: {}", e);
                }
            }
        }
    }
}
