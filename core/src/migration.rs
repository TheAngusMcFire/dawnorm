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

    pub async fn migrate(&self, client: &Client) -> Result<(), Error> {
        let mig_table = client
        .query(r#"SELECT * FROM pg_catalog.pg_tables 
            WHERE schemaname != 'pg_catalog' AND 
            schemaname != 'information_schema' AND 
            tablename = '__dawnorm_schema_migrations';"#, &[])
        .await?;

        if mig_table.is_empty() {
            client.execute(r#"
            CREATE TABLE __dawnorm_schema_migrations (
                name TEXT NOT NULL,
                run_on TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );"#, &[]).await?;
        }

        for migration in &self.migrations {
            let mig = client
                .query("SELECT * FROM __dawnorm_schema_migrations WHERE name = $1", &[&migration.name]).await?;
            if mig.is_empty() {
                client.execute(&migration.up_sql, &[]).await.unwrap();
                client.execute("INSERT INTO __dawnorm_schema_migrations (name) VALUES ($1)", &[&migration.name]).await.unwrap();
            }
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::migration::*;

    #[tokio::test]
    pub async fn migration_test() {
        let migrator = Migrator::new().add_up(
            "initial-migration",
            r#"
        CREATE TABLE test_posts (
            id SERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT
        );"#,
        );

        let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=postgrespw", tokio_postgres::NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        migrator.migrate(&client).await.unwrap();
    }
}