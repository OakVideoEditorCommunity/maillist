use sea_orm::DatabaseConnection;

pub struct ModerationService {
    db: DatabaseConnection,
}

impl ModerationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
