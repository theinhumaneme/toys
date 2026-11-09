mod db;
mod shortner;
mod web;
use db::create_tables;
use rusqlite::{Connection, Result};

#[tokio::main]
async fn main() {
    let conn = connect_to_db().expect("failed to open database");
    let app = web::router(conn);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

fn connect_to_db() -> Result<Connection> {
    let conn = Connection::open("urls.db")?;
    create_tables(&conn)?;
    Ok(conn)
}
