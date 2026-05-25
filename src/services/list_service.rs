use sea_orm::DatabaseConnection;

pub struct ListService {
    db: DatabaseConnection,
}

impl ListService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
