use anyhow::Result;
use axum::serve;
use migration::MigratorTrait;
use oak_maillist::{api::create_router, config::AppConfig, models};
use sea_orm::{Database, EntityTrait, PaginatorTrait};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing(&AppConfig::load()?.logging);

    loop {
        let should_reload = Arc::new(AtomicBool::new(false));

        let result = run_server(should_reload.clone()).await;
        match result {
            Ok(()) => {
                if should_reload.load(Ordering::SeqCst) {
                    info!("Configuration updated, reloading...");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    continue;
                }
                break Ok(());
            }
            Err(e) => break Err(e),
        }
    }
}

async fn run_server(should_reload: Arc<AtomicBool>) -> Result<()> {
    let config = AppConfig::load()?;

    info!("Starting Oak MailList v0.1.0");

    let mut opt = sea_orm::ConnectOptions::new(&config.database.url);
    opt.max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect_timeout(std::time::Duration::from_secs(
            config.database.connect_timeout,
        ))
        .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
        .sqlx_logging(false);
    let db = Database::connect(opt).await?;
    info!("Database connected");

    migration::Migrator::up(&db, None).await?;
    info!("Database migrations applied");

    init_admin_from_env(&db, &config).await;

    let shutdown = Arc::new(Notify::new());
    let app_state =
        models::AppState::new(db.clone(), config.clone(), shutdown.clone(), should_reload);
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
        let smtp_shutdown = shutdown.clone();
        tokio::spawn(async move {
            tokio::select! {
                result = smtp_server.run() => {
                    if let Err(e) = result {
                        error!("SMTP server error: {}", e);
                    }
                }
                _ = smtp_shutdown.notified() => {}
            }
        });
    }

    let digest_state = app_state.clone();
    let digest_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let task = oak_maillist::tasks::digest::DigestTask::new(digest_state.clone());
                    if let Err(e) = task.run().await {
                        error!("Digest task error: {}", e);
                    }
                }
                _ = digest_shutdown.notified() => break,
            }
        }
    });

    let shutdown_signal = async move {
        shutdown.notified().await;
    };

    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await?;

    Ok(())
}

async fn init_admin_from_env(
    db: &sea_orm::DatabaseConnection,
    config: &oak_maillist::config::AppConfig,
) {
    let admin_email = std::env::var("OAK_INIT_ADMIN_EMAIL").ok();
    let admin_password = std::env::var("OAK_INIT_ADMIN_PASSWORD").ok();

    if let (Some(email), Some(password)) = (admin_email, admin_password) {
        let count = oak_maillist::models::user::Entity::find()
            .count(db)
            .await
            .unwrap_or(0);

        if count == 0 {
            let auth_svc =
                oak_maillist::services::auth_service::AuthService::new(db.clone(), config.clone());
            match auth_svc
                .register_admin(&email, &password, Some("Administrator"))
                .await
            {
                Ok(user) => {
                    info!(
                        "Admin account created from environment: {} ({})",
                        user.email, user.id
                    );
                }
                Err(e) => {
                    error!("Failed to create admin from environment: {}", e);
                }
            }
        } else {
            info!("Users already exist, skipping env-init admin creation");
        }
    }
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

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
