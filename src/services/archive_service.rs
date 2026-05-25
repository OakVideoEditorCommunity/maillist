use sea_orm::DatabaseConnection;

pub struct ArchiveService {
    db: DatabaseConnection,
}

impl ArchiveService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
