use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::benchmark::BenchmarkTracker;
use crate::pipeline::RagPipeline;

#[derive(Deserialize)]
pub struct BatchRequest {
    pub file_paths: Vec<String>,
    #[serde(default)]
    pub parallel: bool,
}

#[derive(Serialize)]
pub struct BatchResponse {
    pub success: bool,
    pub results: Vec<DocumentResult>,
    pub summary: BatchSummary,
}

#[derive(Serialize)]
pub struct DocumentResult {
    pub filename: String,
    pub success: bool,
    pub chunks_count: Option<usize>,
    pub error: Option<String>,
    pub parse_ms: Option<String>,
    pub chunk_ms: Option<String>,
}

#[derive(Serialize)]
pub struct BatchSummary {
    pub total_documents: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_ms: String,
    pub parallel: bool,
    pub memory_used_mb: String,
    pub cpu_count: usize,
    pub rayon_threads: usize,
}

pub async fn batch_handler(Json(payload): Json<BatchRequest>) -> impl IntoResponse {
    let mut tracker = BenchmarkTracker::new();
    let pipeline = RagPipeline::with_defaults();

    println!(
        "🚀 Processing {} documents in {} mode",
        payload.file_paths.len(),
        if payload.parallel {
            "parallel"
        } else {
            "sequential"
        }
    );

    let results = if payload.parallel {
        pipeline.process_documents_parallel(&payload.file_paths)
    } else {
        pipeline.process_documents_sequential(&payload.file_paths)
    };

    let mut successful = 0;
    let mut failed = 0;
    let mut document_results = Vec::new();

    for result in results {
        match result {
            Ok(doc) => {
                successful += 1;
                document_results.push(DocumentResult {
                    filename: doc.filename,
                    success: true,
                    chunks_count: Some(doc.chunks.len()),
                    error: None,
                    parse_ms: Some(format!("{:.3}", doc.benchmarks.parse_us as f64 / 1000.0)),
                    chunk_ms: Some(format!("{:.3}", doc.benchmarks.chunk_us as f64 / 1000.0)),
                });
            }
            Err(err) => {
                failed += 1;
                document_results.push(DocumentResult {
                    filename: "unknown".to_string(),
                    success: false,
                    chunks_count: None,
                    error: Some(err),
                    parse_ms: None,
                    chunk_ms: None,
                });
            }
        }
    }

    let resources = tracker.get_resources();

    println!(
        "✅ Batch complete: {}/{} successful in {:.2?}",
        successful,
        payload.file_paths.len(),
        tracker.elapsed()
    );
    println!("💾 Memory used: {} MB", resources.memory_used_mb);
    println!(
        "🖥️  CPUs: {} (Rayon threads: {})\n",
        resources.cpu_count, resources.rayon_threads
    );

    (
        StatusCode::OK,
        Json(BatchResponse {
            success: true,
            results: document_results,
            summary: BatchSummary {
                total_documents: payload.file_paths.len(),
                successful,
                failed,
                total_ms: tracker.elapsed_ms(),
                parallel: payload.parallel,
                memory_used_mb: resources.memory_used_mb,
                cpu_count: resources.cpu_count,
                rayon_threads: resources.rayon_threads,
            },
        }),
    )
}
