use std::sync::Arc;

use tokio_postgres::Client;

use crate::Entity;

#[macro_export]
macro_rules! dbset {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($table_name:ident, $type:ident) => {
        pub fn $table_name(&self) -> DbSet<$type> {DbSet::new(self.client.clone(), stringify!($table_name).into())}
    };
}

pub struct DbSet<T: Entity> {
    client: Arc<Client>,
    phantom:  std::marker::PhantomData<T>,
    table_name: String
}

impl<T: Entity> DbSet<T> {
    pub fn new(client: Arc<Client>, table_name: String) -> Self {
        Self { client, phantom: std::marker::PhantomData, table_name }
    }
}