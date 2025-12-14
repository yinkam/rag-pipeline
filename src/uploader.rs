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

pub struct UploadedFile {
    pub file_path: String,
    pub file_name: String,
    pub size: usize,
}

/// Upload multiple files from a multipart request
pub async fn upload_files(mut multipart: Multipart) -> Result<Vec<UploadedFile>, String> {
    let mut uploaded_files = Vec::new();

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
            return Err(format!("Failed to save file {}: {}", file_name, err));
        }

        println!("Uploaded file: {} ({} bytes)", file_name, data.len());

        uploaded_files.push(UploadedFile {
            file_path,
            file_name,
            size: data.len(),
        });
    }

    if uploaded_files.is_empty() {
        return Err("No files provided".to_string());
    }

    Ok(uploaded_files)
}
