mod batch_handler;
mod benchmark;
mod chunker;
mod handler;
mod parser;
mod pipeline;
mod uploader;

use axum::{
    Router,
    routing::{get, post},
};
use batch_handler::batch_handler;
use handler::request_handler;
use std::fs;

#[tokio::main]
async fn main() {
    // Configure Rayon to use num_cores - 1 threads globally
    let num_threads = (num_cpus::get() - 1).max(1);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();
    println!("🔧 Rayon configured with {} threads\n", num_threads);

    fs::create_dir_all("./uploads").unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/upload", post(request_handler))
        .route("/batch", post(batch_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server running on http://0.0.0.0:3001");
    println!("  POST /upload - Upload and process single file");
    println!("  POST /batch  - Process multiple files in parallel");
    axum::serve(listener, app).await.unwrap();
}
