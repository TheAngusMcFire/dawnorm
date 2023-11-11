use tokio_postgres::Client;

use crate::Error;


pub struct Migration {
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>
}

pub struct Migrator {
    pub migrations: Vec<Migration>
}

impl Migrator {
    pub fn new() -> Self {
        Self { migrations: Vec::new() }
    }
    pub fn add_up(mut self, name: &str, up_sql: &str) -> Self{
        self.migrations.push(Migration { name: name.into(), up_sql: up_sql.into(), down_sql: None });
        self
    }

    pub fn add_up_down(mut self, name: &str, up_sql: &str, down_sql: &str) -> Self{
        self.migrations.push(Migration { name: name.into(), up_sql: up_sql.into(), down_sql: Some(down_sql.into()) });
        self
    }

    pub async fn migrate(client: &Client) -> Result<(), Error> {
        todo!()
    }
}