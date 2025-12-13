use kiru::{BytesChunker, Chunker};

pub fn chunk_text(
    text: &str,
    chunk_size: usize,
    overlap: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let chunker = BytesChunker::new(chunk_size, overlap)?;

    let chunks: Vec<String> = chunker.chunk_string(text.to_string()).collect();

    Ok(chunks)
}
