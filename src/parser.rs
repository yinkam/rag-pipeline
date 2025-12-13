use extractous::Extractor;
use extractous::PdfParserConfig;
use std::collections::HashMap;

pub fn parse_pdf(
    file_path: &str,
) -> Result<(String, HashMap<String, Vec<String>>), Box<dyn std::error::Error>> {
    let mut extractor = Extractor::new().set_extract_string_max_length(100000);

    let custom_pdf_config = true;

    if custom_pdf_config {
        extractor =
            extractor.set_pdf_config(PdfParserConfig::new().set_extract_annotation_text(false));
    }

    extractor
        .extract_file_to_string(file_path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
