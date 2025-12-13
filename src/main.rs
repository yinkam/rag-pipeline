use axum::{
    Router,
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::{fs, path::Path};

#[tokio::main]
async fn main() {
    fs::create_dir_all("./uploads").unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/upload", post(upload_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server running on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
fn sanitize_filename(filename: &str) -> String {
    Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
        .replace(
            |c: char| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'),
            "_",
        )
}

async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => sanitize_filename(name),
            None => continue, // Skip non-file fields
        };

        if file_name.is_empty() {
            continue;
        }

        let data = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("Error reading file data: {:?}", err);
                return (StatusCode::BAD_REQUEST, "Failed to read file data");
            }
        };

        let file_path = format!("./uploads/{}", file_name);
        if let Err(err) = fs::write(&file_path, &data) {
            eprintln!("Error writing file: {:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file");
        }

        println!("Uploaded file: {} ({} bytes)", file_name, data.len());
        return (StatusCode::OK, "File uploaded successfully");
    }

    (StatusCode::BAD_REQUEST, "No file provided")
}
