use rayon::prelude::*;
use std::path::Path;
use std::time::Instant;

use crate::chunker::chunk_text;
use crate::parser::parse_pdf;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub chunk_size: usize,
    pub overlap: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024,
            overlap: 128,
        }
    }
}

#[derive(Debug)]
pub struct PipelineResult {
    pub filename: String,
    pub chunks: Vec<String>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
    pub benchmarks: PipelineBenchmarks,
}

#[derive(Debug)]
pub struct PipelineBenchmarks {
    pub parse_us: u128,
    pub chunk_us: u128,
    pub total_us: u128,
}

pub struct RagPipeline {
    config: PipelineConfig,
}

impl RagPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// Process a single document through the pipeline
    pub fn process_document(&self, file_path: &str) -> Result<PipelineResult, String> {
        let total_start = Instant::now();

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path)
            .to_string();

        // Parse stage
        let parse_start = Instant::now();
        let (parsed_text, metadata) =
            parse_pdf(file_path).map_err(|e| format!("Failed to parse PDF: {}", e))?;
        let parse_us = parse_start.elapsed().as_micros();

        // Chunking stage
        let chunk_start = Instant::now();
        let chunks = chunk_text(&parsed_text, self.config.chunk_size, self.config.overlap)
            .map_err(|e| format!("Failed to chunk text: {}", e))?;
        let chunk_us = chunk_start.elapsed().as_micros();

        let total_us = total_start.elapsed().as_micros();

        Ok(PipelineResult {
            filename,
            chunks,
            metadata,
            benchmarks: PipelineBenchmarks {
                parse_us,
                chunk_us,
                total_us,
            },
        })
    }

    /// Process multiple documents in parallel using Rayon
    pub fn process_documents_parallel(
        &self,
        file_paths: &[String],
    ) -> Vec<Result<PipelineResult, String>> {
        file_paths
            .par_iter()
            .map(|path| self.process_document(path))
            .collect()
    }

    /// Process multiple documents sequentially
    pub fn process_documents_sequential(
        &self,
        file_paths: &[String],
    ) -> Vec<Result<PipelineResult, String>> {
        file_paths
            .iter()
            .map(|path| self.process_document(path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = RagPipeline::with_defaults();
        assert_eq!(pipeline.config.chunk_size, 1024);
        assert_eq!(pipeline.config.overlap, 128);
    }

    #[test]
    fn test_custom_config() {
        let config = PipelineConfig {
            chunk_size: 512,
            overlap: 64,
        };
        let pipeline = RagPipeline::new(config);
        assert_eq!(pipeline.config.chunk_size, 512);
        assert_eq!(pipeline.config.overlap, 64);
    }
}
