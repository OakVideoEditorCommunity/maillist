mod common;
use common::setup_db;
use oak_maillist::services::template_service::TemplateService;
use sea_orm::{ActiveModelTrait, Set};

async fn seed_template(db: &sea_orm::DatabaseConnection, name: &str, subject: &str, body_text: Option<&str>, body_html: Option<&str>) {
    let tmpl = oak_maillist::models::email_template::ActiveModel {
        id: Set(oak_maillist::utils::crypto::generate_uuid()),
        name: Set(name.to_string()),
        subject: Set(Some(subject.to_string())),
        body_text: Set(body_text.map(|s| s.to_string())),
        body_html: Set(body_html.map(|s| s.to_string())),
        variables: Set(None),
        is_system: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    tmpl.insert(db).await.unwrap();
}

#[tokio::test]
async fn test_render_template_not_found() {
    let state = setup_db().await;
    let svc = TemplateService::new(state.db.clone());
    let ctx = tera::Context::new();
    let result = svc.render_template("nonexistent", &ctx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_render_template_with_variables() {
    let state = setup_db().await;
    let svc = TemplateService::new(state.db.clone());
    
    let mut ctx = tera::Context::new();
    ctx.insert("list_name", "Test List");
    ctx.insert("subscriber_name", "Alice");
    let (subject, body) = svc.render_template("welcome", &ctx).await.unwrap();
    
    assert_eq!(subject, Some("欢迎加入 Test List".to_string()));
    assert!(body.as_ref().unwrap().contains("Alice"));
}

#[tokio::test]
async fn test_render_template_html_fallback() {
    let state = setup_db().await;
    seed_template(&state.db, "html_only", "Subject", None, Some("<html>Hi</html>")).await;
    let svc = TemplateService::new(state.db.clone());
    
    let ctx = tera::Context::new();
    let (subject, body) = svc.render_template("html_only", &ctx).await.unwrap();
    
    assert_eq!(subject, Some("Subject".to_string()));
    assert_eq!(body, Some("<html>Hi</html>".to_string()));
}

#[tokio::test]
async fn test_render_template_no_body() {
    let state = setup_db().await;
    seed_template(&state.db, "empty", "Subject", None, None).await;
    let svc = TemplateService::new(state.db.clone());
    
    let ctx = tera::Context::new();
    let result = svc.render_template("empty", &ctx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_send_templated_email_skips_when_no_smtp() {
    let state = setup_db().await;
    seed_template(&state.db, "notify", "Hello", Some("Body"), None).await;
    let svc = TemplateService::new(state.db.clone());
    
    let ctx = tera::Context::new();
    let smtp = oak_maillist::config::SmtpOutgoingConfig {
        host: "".to_string(),
        port: 587,
        username: "".to_string(),
        password: "".to_string(),
        from_address: "test@example.com".to_string(),
    };
    let result = svc.send_templated_email("to@example.com", "notify", &ctx, &smtp).await;
    assert!(result.is_ok());
}
