use sea_orm::DatabaseConnection;

pub struct SubscriberService {
    db: DatabaseConnection,
}

impl SubscriberService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
