use sea_orm::DatabaseConnection;

pub struct MailService {
    db: DatabaseConnection,
}

impl MailService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
