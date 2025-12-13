mod chunker;
mod handler;
mod parser;
mod uploader;

use axum::{
    Router,
    routing::{get, post},
};
use handler::request_handler;
use std::fs;

#[tokio::main]
async fn main() {
    fs::create_dir_all("./uploads").unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/upload", post(request_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server running on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
