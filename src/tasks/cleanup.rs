use sea_orm::DatabaseConnection;

pub struct CleanupTask {
    db: DatabaseConnection,
}

impl CleanupTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
