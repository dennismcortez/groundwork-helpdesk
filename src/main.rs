mod db;

use axum::{extract::State, routing::get, Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() {
    let pool = db::init_pool().await.expect("Failed to initialize database");
    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(|| async { "Hello from Rust!" }))
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
