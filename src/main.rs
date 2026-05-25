use anyhow::Result;
use axum::serve;
use oak_maillist::{api::create_router, config::AppConfig, models};
use sea_orm::Database;
use migration::MigratorTrait;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    init_tracing(&config.logging);

    info!("Starting Oak MailList v0.1.0");
    info!("Loaded configuration from: config/default.toml");

    let db = Database::connect(&config.database.url).await?;
    info!("Database connected");

    migration::Migrator::up(&db, None).await?;
    info!("Database migrations applied");

    let app_state = models::AppState { db: db.clone(), config: config.clone() };
    let app = create_router(app_state.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);

    if config.smtp.incoming.enabled {
        let smtp_server = oak_maillist::smtp::server::SmtpServer::new(
            config.smtp.incoming.host.clone(),
            config.smtp.incoming.port,
            app_state.clone(),
        );
        tokio::spawn(async move {
            if let Err(e) = smtp_server.run().await {
                error!("SMTP server error: {}", e);
            }
        });
    }

    serve(listener, app).await?;

    Ok(())
}

fn init_tracing(logging: &oak_maillist::config::LoggingConfig) {
    let level = match logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
