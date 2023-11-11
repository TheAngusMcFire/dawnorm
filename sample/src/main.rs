use dawnorm::context::*;
use std::sync::Arc;

use tokio_postgres::{Client, NoTls};

#[derive(dawnorm_codegen::Entity)]
pub struct Post {
    id: i32,
    title: String,
    body: Option<String>,
}

pub struct DbContex {
    client: Arc<Client>,
}

impl DbContex {
    dawnorm::dbset!(posts, Post);
}

#[tokio::main]
async fn main() {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=postgrespw", NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let ctx = DbContex{ client: client.into() };
        

}
