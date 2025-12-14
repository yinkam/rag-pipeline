use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::time::Instant;

use crate::benchmark::BenchmarkTracker;
use crate::pipeline::RagPipeline;
use crate::uploader::upload_file;

pub async fn request_handler(multipart: Multipart) -> impl IntoResponse {
    let mut tracker = BenchmarkTracker::new();

    // Upload stage
    let upload_start = Instant::now();
    let (file_path, data) = match upload_file(multipart).await {
        Ok(result) => result,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": err })));
        }
    };
    let upload_duration = upload_start.elapsed();
    println!("⏱️  Upload: {:.2?}", upload_duration);

    // Process through pipeline
    let pipeline = RagPipeline::with_defaults();
    let result = match pipeline.process_document(&file_path) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Pipeline error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err })),
            );
        }
    };

    println!(
        "⏱️  Parse: {:.2?}",
        std::time::Duration::from_micros(result.benchmarks.parse_us as u64)
    );
    println!(
        "⏱️  Chunk: {:.2?} ({} chunks generated)",
        std::time::Duration::from_micros(result.benchmarks.chunk_us as u64),
        result.chunks.len()
    );

    tracker.print_summary("Total");

    let resources = tracker.get_resources();

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "filename": result.filename,
            "size": data.len(),
            "text_chunks": result.chunks,
            "metadata": result.metadata,
            "benchmarks": {
                "upload_us": upload_duration.as_micros(),
                "parse_us": result.benchmarks.parse_us,
                "chunk_us": result.benchmarks.chunk_us,
                "total_us": tracker.elapsed_micros(),
                "upload_ms": format!("{:.3}", upload_duration.as_secs_f64() * 1000.0),
                "parse_ms": format!("{:.3}", result.benchmarks.parse_us as f64 / 1000.0),
                "chunk_ms": format!("{:.3}", result.benchmarks.chunk_us as f64 / 1000.0),
                "total_ms": tracker.elapsed_ms()
            },
            "resources": resources
        })),
    )
}
