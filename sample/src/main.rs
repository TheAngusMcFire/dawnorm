mod migrations;

use dawnorm::{context::*, parms};
use std::sync::Arc;

use tokio_postgres::{Client, NoTls};

#[derive(dawnorm_codegen::Entity, Debug)]
pub struct Post {
    #[key_noinsert_noupdate]
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

    migrations::build_migrator().migrate(&client).await.unwrap();

    let ctx = DbContex { client: client.into() };
    let p = ctx.posts()
        .filter(format!("{} = $1 or {} = $2", PostFields::title(), PostFields::id()), parms!["test", 4])
        .order_by(PostFields::id(), Ordering::ASC)
        //.skip(3)
        //.take(1)
        .to_vec().await.unwrap();
    dbg!(p);
    
    let i = Post { id: 0, title: "this is cool".into(), body: Some("this is the body".into()) };
    let ri = ctx.posts().insert(i).await.unwrap();
    dbg!(&ri);
    
    let mut to_update = ctx.posts().filter_pk(parms!(5)).first().await.unwrap();
    dbg!(&to_update);
    to_update.body = Some(format!("updated + {}", &to_update.body.unwrap()));
    let updated = ctx.posts().update(to_update).await.unwrap();
    dbg!(updated);

    ctx.posts().delete(&ri).await.unwrap();
}
 