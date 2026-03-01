//! Source location & line/column mapping.
//!
//! Mirrors the `source_mapping` block in upstream Slither's JSON output, plus
//! the columns/lines derivation logic used by `evm` and `slithir` printers.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A location inside a source file — byte offset + derived line/column data.
///
/// Matches upstream Slither's `source_mapping` JSON record shape so that
/// `--json` output is byte-equivalent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Absolute path on disk, or empty if synthesized.
    pub filename_absolute: PathBuf,
    /// Path relative to the project root.
    pub filename_relative: PathBuf,
    /// Path as it appears in import statements / artifacts.
    pub filename_used: PathBuf,
    /// Short display name (just the file basename).
    pub filename_short: String,
    /// Byte offset of the start of the span.
    pub start: usize,
    /// Span length in bytes.
    pub length: usize,
    /// 1-indexed source line(s) covered by the span.
    pub lines: Vec<usize>,
    /// 1-indexed column of the first byte.
    pub starting_column: usize,
    /// 1-indexed column of the last byte.
    pub ending_column: usize,
    /// Compilation unit ID (used by multi-unit projects).
    #[serde(default)]
    pub compilation_unit: Option<String>,
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self {
            filename_absolute: PathBuf::new(),
            filename_relative: PathBuf::new(),
            filename_used: PathBuf::new(),
            filename_short: String::new(),
            start: 0,
            length: 0,
            lines: Vec::new(),
            starting_column: 0,
            ending_column: 0,
            compilation_unit: None,
        }
    }
}

impl SourceLocation {
    /// Build a `SourceLocation` from a solc compact source-map triplet
    /// `start:length:file_index` plus the source file content.
    pub fn from_solc_triplet(
        triplet: &str,
        file_path: &Path,
        relative_to: Option<&Path>,
        content: &str,
    ) -> Option<Self> {
        let mut parts = triplet.split(':');
        let start: usize = parts.next()?.parse().ok()?;
        let length: usize = parts.next()?.parse().ok()?;
        let _file_id: i32 = parts.next()?.parse().ok()?;
        Some(Self::from_span(start, length, file_path, relative_to, content))
    }

    /// Build a `SourceLocation` from explicit byte offsets + the source content
    /// to derive line/column info.
    pub fn from_span(
        start: usize,
        length: usize,
        file_path: &Path,
        relative_to: Option<&Path>,
        content: &str,
    ) -> Self {
        let (lines, start_col, end_col) = compute_lines_columns(content, start, length);
        let filename_short = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let filename_relative = relative_to
            .and_then(|root| file_path.strip_prefix(root).ok())
            .map_or_else(|| file_path.to_path_buf(), Path::to_path_buf);
        Self {
            filename_absolute: file_path.to_path_buf(),
            filename_relative,
            filename_used: file_path.to_path_buf(),
            filename_short,
            start,
            length,
            lines,
            starting_column: start_col,
            ending_column: end_col,
            compilation_unit: None,
        }
    }

    /// Build a minimal location for a whole file (no offsets known).
    pub fn whole_file(file_path: &Path, relative_to: Option<&Path>) -> Self {
        let filename_short = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let filename_relative = relative_to
            .and_then(|root| file_path.strip_prefix(root).ok())
            .map_or_else(|| file_path.to_path_buf(), Path::to_path_buf);
        Self {
            filename_absolute: file_path.to_path_buf(),
            filename_relative,
            filename_used: file_path.to_path_buf(),
            filename_short,
            start: 0,
            length: 0,
            lines: vec![1],
            starting_column: 1,
            ending_column: 1,
            compilation_unit: None,
        }
    }

    /// Format the location like `Contract.sol#L12-L15`.
    pub fn pretty(&self) -> String {
        if self.lines.is_empty() {
            self.filename_short.clone()
        } else if self.lines.len() == 1 {
            format!("{}#L{}", self.filename_short, self.lines[0])
        } else {
            format!(
                "{}#L{}-L{}",
                self.filename_short,
                self.lines.first().copied().unwrap_or(0),
                self.lines.last().copied().unwrap_or(0)
            )
        }
    }
}

fn compute_lines_columns(content: &str, start: usize, length: usize) -> (Vec<usize>, usize, usize) {
    if content.is_empty() {
        return (Vec::new(), 0, 0);
    }
    let end = start.saturating_add(length).min(content.len());
    let bytes = content.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let mut start_line = 0usize;
    let mut start_col = 0usize;
    let mut end_line = 0usize;
    let mut end_col = 0usize;
    let mut lines: Vec<usize> = Vec::new();
    let mut last_pushed = 0usize;
    let upper = bytes.len();
    let real_start = start.min(upper);
    let real_end = end.min(upper);
    for i in 0..upper {
        if i == real_start {
            start_line = line;
            start_col = col;
            if last_pushed != line {
                lines.push(line);
                last_pushed = line;
            }
        }
        if i >= real_start && i < real_end {
            if last_pushed != line {
                lines.push(line);
                last_pushed = line;
            }
        }
        if i == real_end.saturating_sub(1) {
            end_line = line;
            end_col = col;
            break;
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    if start_line == 0 {
        start_line = line;
        start_col = col;
    }
    if end_line == 0 {
        end_line = start_line;
        end_col = start_col;
    }
    if lines.is_empty() {
        lines.push(start_line);
    }
    let _ = end_line;
    (lines, start_col, end_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_span_basic() {
        let src = "line1\nline2 hello\nline3\n";
        let loc = SourceLocation::from_span(6, 11, Path::new("file.sol"), None, src);
        assert_eq!(loc.filename_short, "file.sol");
        assert_eq!(loc.lines, vec![2]);
        assert_eq!(loc.starting_column, 1);
    }

    #[test]
    fn whole_file_defaults() {
        let loc = SourceLocation::whole_file(Path::new("X.sol"), None);
        assert_eq!(loc.filename_short, "X.sol");
        assert_eq!(loc.lines, vec![1]);
    }
}
