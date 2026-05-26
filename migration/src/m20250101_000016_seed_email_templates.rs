use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let now = chrono::Utc::now().to_rfc3339();

        let templates = vec![
            (
                uuid::Uuid::new_v4(),
                "subscription_confirm",
                "请确认您的订阅 - {{ list_name }}",
                "您好 {{ subscriber_name | default(value=\"\") }},\n\n请点击以下链接确认订阅 {{ list_name }}:\n{{ confirm_url }}\n\n如非您本人操作，请忽略此邮件。",
                "<p>您好 {{ subscriber_name | default(value=\"\") }},</p><p>请点击以下链接确认订阅 <strong>{{ list_name }}</strong>:</p><p><a href=\"{{ confirm_url }}\">{{ confirm_url }}</a></p><p>如非您本人操作，请忽略此邮件。</p>",
            ),
            (
                uuid::Uuid::new_v4(),
                "unsubscribe_confirm",
                "请确认您的退订 - {{ list_name }}",
                "您好 {{ subscriber_name | default(value=\"\") }},\n\n请点击以下链接确认退订 {{ list_name }}:\n{{ confirm_url }}\n\n如非您本人操作，请忽略此邮件。",
                "<p>您好 {{ subscriber_name | default(value=\"\") }},</p><p>请点击以下链接确认退订 <strong>{{ list_name }}</strong>:</p><p><a href=\"{{ confirm_url }}\">{{ confirm_url }}</a></p><p>如非您本人操作，请忽略此邮件。</p>",
            ),
            (
                uuid::Uuid::new_v4(),
                "moderation_notice",
                "待审核邮件 - {{ list_name }}",
                "您好管理员,\n\n邮件列表 {{ list_name }} 收到一封需要审核的邮件:\n主题: {{ message_subject }}\n\n请前往审核:\n{{ review_url }}",
                "<p>您好管理员,</p><p>邮件列表 <strong>{{ list_name }}</strong> 收到一封需要审核的邮件:</p><p>主题: {{ message_subject }}</p><p>请前往审核:</p><p><a href=\"{{ review_url }}\">{{ review_url }}</a></p>",
            ),
            (
                uuid::Uuid::new_v4(),
                "welcome",
                "欢迎加入 {{ list_name }}",
                "您好 {{ subscriber_name | default(value=\"\") }},\n\n欢迎加入 {{ list_name }}!\n\n您将收到此列表的所有邮件。如需更改接收方式或退订，请访问列表设置页面。",
                "<p>您好 {{ subscriber_name | default(value=\"\") }},</p><p>欢迎加入 <strong>{{ list_name }}</strong>!</p><p>您将收到此列表的所有邮件。如需更改接收方式或退订，请访问列表设置页面。</p>",
            ),
        ];

        for (id, name, subject, body_text, body_html) in templates {
            let id_hex = id.as_simple().to_string();
            let sql = format!(
                "INSERT INTO email_template (id, name, subject, body_text, body_html, is_system, created_at, updated_at)
                 VALUES (X'{}', '{}', '{}', '{}', '{}', true, '{}', '{}')
                 ON CONFLICT (name) DO NOTHING",
                id_hex, name,
                subject.replace("'", "''"),
                body_text.replace("'", "''"),
                body_html.replace("'", "''"),
                now, now
            );
            db.execute_unprepared(&sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DELETE FROM email_template WHERE name IN ('subscription_confirm','unsubscribe_confirm','moderation_notice','welcome')"
        )
        .await?;
        Ok(())
    }
}
