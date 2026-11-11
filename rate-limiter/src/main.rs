mod algos;
mod limiter;
mod middleware;
mod web;

#[tokio::main]
async fn main() {
    let app = web::router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
