pub mod context;
pub mod migration;

use tokio_postgres::Row;


#[derive(Debug)]
pub enum Error {
    TokioPostgres(tokio_postgres::Error)
}

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        Error::TokioPostgres(err)
    }
}

pub struct EntityFieldDefinition {
    pub name: String,
    pub optional: bool,
    pub psql_type: String
}


pub trait Entity: Sized {
    fn from_row(row: Row) -> Result<Self, Error>;
    fn sql_fields() -> &'static str;
    fn sql_table_fields(table_name: &str) -> String;
    fn entity_fields() -> Vec<EntityFieldDefinition>;
    fn primary_key_name() -> &'static str;
    fn get_insert_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>);
    fn get_update_query(self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>);
    fn get_delete_query(&self, table_name: &str) -> (String, Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>);
}
