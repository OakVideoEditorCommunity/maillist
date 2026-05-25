use sea_orm::DatabaseConnection;

pub struct DigestTask {
    db: DatabaseConnection,
}

impl DigestTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
