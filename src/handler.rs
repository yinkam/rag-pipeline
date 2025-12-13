use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::chunker::chunk_text;
use crate::parser::parse_pdf;
use crate::uploader::upload_file;

pub async fn request_handler(multipart: Multipart) -> impl IntoResponse {
    let (file_path, data) = match upload_file(multipart).await {
        Ok(result) => result,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err })));
        }
    };

    let file_name = file_path.trim_start_matches("./uploads/");

    // Parse the PDF
    let (parsed_text, metadata) = match parse_pdf(&file_path) {
        Ok((text, metadata)) => {
            println!("Extracted {} characters of text", text.len());
            (text, metadata)
        }
        Err(err) => {
            eprintln!("Error parsing PDF: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to parse PDF",
                    "filename": file_name
                })),
            );
        }
    };

    match chunk_text(&parsed_text, 1024, 128) {
        Ok(chunks) => {
            println!("Generated {} text chunks", chunks.len());
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "filename": file_name,
                    "size": data.len(),
                    "text_chunks": chunks,
                    "metadata": metadata
                })),
            )
        }
        Err(err) => {
            eprintln!("Error chunking text: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to chunk text",
                    "filename": file_name
                })),
            )
        }
    }
}
