use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use serde_json::json;

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
    match parse_pdf(&file_path) {
        Ok((text, metadata)) => {
            println!("Extracted {} characters of text", text.len());
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "filename": file_name,
                    "size": data.len(),
                    "text": text,
                    "metadata": metadata
                })),
            )
        }
        Err(err) => {
            eprintln!("Error parsing PDF: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to parse PDF",
                    "filename": file_name
                })),
            )
        }
    }
}
