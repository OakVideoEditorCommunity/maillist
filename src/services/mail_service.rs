use sea_orm::DatabaseConnection;

pub struct MailService {
    _db: DatabaseConnection,
}

impl MailService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { _db: db }
    }
}
