use axum::extract::Multipart;
use std::fs;
use std::path::Path;

pub fn sanitize_filename(filename: &str) -> String {
    Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
        .replace(
            |c: char| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'),
            "_",
        )
}

pub async fn upload_file(mut multipart: Multipart) -> Result<(String, Vec<u8>), String> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => sanitize_filename(name),
            None => continue, // Skip non-file fields
        };

        if file_name.is_empty() {
            continue;
        }

        let data = match field.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(err) => {
                eprintln!("Error reading file data: {:?}", err);
                return Err("Failed to read file data".to_string());
            }
        };

        let file_path = format!("./uploads/{}", file_name);
        if let Err(err) = fs::write(&file_path, &data) {
            eprintln!("Error writing file: {:?}", err);
            return Err("Failed to save file".to_string());
        }

        println!("Uploaded file: {} ({} bytes)", file_name, data.len());
        return Ok((file_path, data));
    }

    Err("No file provided".to_string())
}
