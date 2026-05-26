use crate::models::AppState;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

pub struct SmtpServer {
    host: String,
    port: u16,
    state: AppState,
}

#[derive(Debug, Clone)]
pub struct IncomingEmail {
    pub from: String,
    pub to: Vec<String>,
    pub raw_data: Vec<u8>,
    pub remote_addr: String,
}

impl SmtpServer {
    pub fn new(host: String, port: u16, state: AppState) -> Self {
        Self { host, port, state }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("SMTP server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, peer.to_string(), state).await {
                            warn!("SMTP connection error from {}: {}", peer, e);
                        }
                    });
                }
                Err(e) => {
                    error!("SMTP accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    remote_addr: String,
    state: AppState,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    writer.write_all(b"220 Oak MailList ESMTP\r\n").await?;

    let mut session = SmtpSession {
        state,
        remote_addr,
        from: None,
        to: Vec::new(),
        data: Vec::new(),
        in_data: false,
    };

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let cmd = line.trim();
        info!("SMTP command from {}: {}", session.remote_addr, cmd);

        if session.in_data {
            if cmd == "." {
                session.in_data = false;

                let email = IncomingEmail {
                    from: session.from.clone().unwrap_or_default(),
                    to: session.to.clone(),
                    raw_data: session.data.clone(),
                    remote_addr: session.remote_addr.clone(),
                };

                if let Err(e) = process_email(email, &session.state).await {
                    error!("Failed to process email: {}", e);
                    writer
                        .write_all(b"451 Requested action aborted: local error\r\n")
                        .await?;
                } else {
                    writer.write_all(b"250 OK\r\n").await?;
                }

                session.data.clear();
                session.from = None;
                session.to.clear();
            } else {
                if cmd.starts_with("..") {
                    session.data.extend_from_slice(&cmd[1..].as_bytes());
                } else {
                    session.data.extend_from_slice(cmd.as_bytes());
                }
                session.data.extend_from_slice(b"\r\n");
            }
            continue;
        }

        let response = if cmd.len() >= 4 {
            let command = cmd[..4].to_uppercase();
            match command.as_str() {
                "EHLO" | "HELO" => "250-Ok\r\n250-SIZE 10485760\r\n250 PIPELINING\r\n".to_string(),
                "MAIL" => {
                    if let Some(addr) = extract_address(cmd) {
                        session.from = Some(addr);
                        "250 OK\r\n".to_string()
                    } else {
                        "501 Syntax error in parameters\r\n".to_string()
                    }
                }
                "RCPT" => {
                    if let Some(addr) = extract_address(cmd) {
                        session.to.push(addr);
                        "250 OK\r\n".to_string()
                    } else {
                        "501 Syntax error in parameters\r\n".to_string()
                    }
                }
                "DATA" => {
                    session.in_data = true;
                    "354 End data with <CR><LF>.<CR><LF>\r\n".to_string()
                }
                "QUIT" => {
                    writer.write_all(b"221 Bye\r\n").await?;
                    break;
                }
                "RSET" => {
                    session.from = None;
                    session.to.clear();
                    session.data.clear();
                    "250 OK\r\n".to_string()
                }
                "NOOP" => "250 OK\r\n".to_string(),
                _ => "500 Command not recognized\r\n".to_string(),
            }
        } else {
            "500 Command not recognized\r\n".to_string()
        };

        writer.write_all(response.as_bytes()).await?;
    }

    Ok(())
}

struct SmtpSession {
    state: AppState,
    remote_addr: String,
    from: Option<String>,
    to: Vec<String>,
    data: Vec<u8>,
    in_data: bool,
}

fn extract_address(cmd: &str) -> Option<String> {
    let start = cmd.find('<')?;
    let end = cmd.find('>')?;
    if start < end {
        Some(cmd[start + 1..end].to_string())
    } else {
        None
    }
}

async fn process_email(email: IncomingEmail, state: &AppState) -> anyhow::Result<()> {
    info!(
        "Processing email from {} to {:?}, size: {} bytes",
        email.from,
        email.to,
        email.raw_data.len()
    );

    let raw_data = email.raw_data.clone();
    let parsed = crate::smtp::parser::EmailParser::parse(&raw_data)?;

    let pipeline = crate::smtp::processor::MailPipeline::new(state.clone());
    pipeline.process(email, parsed).await?;

    Ok(())
}
