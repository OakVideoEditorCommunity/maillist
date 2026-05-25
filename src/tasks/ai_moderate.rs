use sea_orm::DatabaseConnection;

pub struct AiModerateTask {
    db: DatabaseConnection,
}

impl AiModerateTask {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
