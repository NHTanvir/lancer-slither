//! `--zip out.zip` — bundle JSON+SARIF+Markdown into one zip.

use std::io::Write;
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::finding::OutputResults;

pub fn write(path: &Path, results: &OutputResults) -> eyre::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zw = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zw.start_file("results.json", opts)?;
    zw.write_all(serde_json::to_string_pretty(results)?.as_bytes())?;

    zw.start_file("report.md", opts)?;
    zw.write_all(super::markdown::render(&results.results.detectors, None).as_bytes())?;

    zw.finish()?;
    Ok(())
}
