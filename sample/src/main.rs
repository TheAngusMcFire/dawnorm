mod migrations;

use dawnorm::{context::*, parms};
use std::sync::Arc;

use tokio_postgres::{Client, NoTls, types::ToSql};

#[derive(dawnorm_codegen::Entity, Debug)]
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
    color_eyre::install().unwrap();
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=postgrespw", NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let ctx = DbContex { client: client.into() };
    let p = ctx.posts()
        .filter("title = $1 or id = $2", parms!["test", 4])
        .order_by("title", Ordering::DESC)
        //.skip(3)
        .take(1)
        .to_vec().await.unwrap();
    dbg!(p);
    
    let i = Post { id: 0, title: "this is cool".into(), body: Some("this is the body".into()) };
    let ri = ctx.posts().insert(i).await.unwrap();
    dbg!(&ri);

    let mut to_update = ctx.posts().filter("id=$1", parms![10]).first()
        .await.unwrap().unwrap();
    dbg!(&to_update);
    to_update.body = Some("this was updated".into());
    let updated = ctx.posts().update(to_update).await.unwrap();
    dbg!(updated);

    let del = ctx.posts().delete(&ri).await.unwrap();
    dbg!(del);
    
}
