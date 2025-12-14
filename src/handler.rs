use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;
use std::time::Instant;

use crate::benchmark::BenchmarkTracker;
use crate::pipeline::RagPipeline;
use crate::uploader::upload_files;

#[derive(Serialize)]
pub struct DocumentResult {
    pub filename: String,
    pub success: bool,
    pub size: usize,
    pub chunks_count: Option<usize>,
    pub error: Option<String>,
    pub upload_ms: Option<String>,
    pub parse_ms: Option<String>,
    pub chunk_ms: Option<String>,
    pub total_ms: Option<String>,
}

pub async fn request_handler(multipart: Multipart) -> impl IntoResponse {
    let mut tracker = BenchmarkTracker::new();

    // Upload stage
    let upload_start = Instant::now();
    let uploaded_files = match upload_files(multipart).await {
        Ok(files) => files,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err })));
        }
    };
    let upload_duration = upload_start.elapsed();
    println!(
        "⏱️  Upload: {:.2?} ({} files)",
        upload_duration,
        uploaded_files.len()
    );

    // Process through pipeline (parallel if multiple files)
    let pipeline = RagPipeline::with_defaults();
    let file_paths: Vec<String> = uploaded_files.iter().map(|f| f.file_path.clone()).collect();

    let results = if file_paths.len() > 1 {
        println!("🚀 Processing {} files in parallel", file_paths.len());
        pipeline.process_documents_parallel(&file_paths)
    } else {
        vec![pipeline.process_document(&file_paths[0])]
    };

    let mut document_results = Vec::new();
    let upload_ms_per_file = format!(
        "{:.3}",
        upload_duration.as_secs_f64() * 1000.0 / uploaded_files.len() as f64
    );

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(doc) => {
                println!(
                    "  ✓ {}: {} chunks ({:.2?} parse, {:.2?} chunk)",
                    doc.filename,
                    doc.chunks.len(),
                    std::time::Duration::from_micros(doc.benchmarks.parse_us as u64),
                    std::time::Duration::from_micros(doc.benchmarks.chunk_us as u64)
                );

                let total_ms = (doc.benchmarks.parse_us + doc.benchmarks.chunk_us) as f64 / 1000.0;

                document_results.push(DocumentResult {
                    filename: doc.filename,
                    success: true,
                    size: uploaded_files[i].size,
                    chunks_count: Some(doc.chunks.len()),
                    error: None,
                    upload_ms: Some(upload_ms_per_file.clone()),
                    parse_ms: Some(format!("{:.3}", doc.benchmarks.parse_us as f64 / 1000.0)),
                    chunk_ms: Some(format!("{:.3}", doc.benchmarks.chunk_us as f64 / 1000.0)),
                    total_ms: Some(format!("{:.3}", total_ms)),
                });
            }
            Err(err) => {
                eprintln!("  ✗ Error: {}", err);
                document_results.push(DocumentResult {
                    filename: uploaded_files
                        .get(i)
                        .map(|f| f.file_name.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    success: false,
                    size: uploaded_files.get(i).map(|f| f.size).unwrap_or(0),
                    chunks_count: None,
                    error: Some(err),
                    upload_ms: Some(upload_ms_per_file.clone()),
                    parse_ms: None,
                    chunk_ms: None,
                    total_ms: None,
                });
            }
        }
    }

    tracker.print_summary("Total");
    let resources = tracker.get_resources();

    let successful = document_results.iter().filter(|d| d.success).count();
    let failed = document_results.len() - successful;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "results": document_results,
            "summary": {
                "total_files": uploaded_files.len(),
                "successful": successful,
                "failed": failed,
                "total_ms": tracker.elapsed_ms(),
                "upload_ms": format!("{:.3}", upload_duration.as_secs_f64() * 1000.0),
            },
            "resources": resources
        })),
    )
}
