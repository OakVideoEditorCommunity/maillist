use sea_orm::DatabaseConnection;

pub struct DeliverTask {
    db: DatabaseConnection,
}

impl DeliverTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
