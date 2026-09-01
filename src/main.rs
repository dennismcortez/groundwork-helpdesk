use axum::{routing::get, Router};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Placeholder route – will be replaced with real handlers later
    let app = Router::new()
        .route("/", get(|| async { "Hello from Rust!" }))
        .fallback_service(ServeDir::new("public"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
