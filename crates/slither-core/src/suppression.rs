//! `slither-disable-*` annotation parser.
//!
//! Supported forms:
//! * `// slither-disable-next-line check1,check2`
//! * `// slither-disable-start check1`
//! * `// slither-disable-end check1`
//! * `// slither-disable-line check1` (alias for next-line on the same line)
//!
//! When `--show-ignored-findings` is set we still emit the suppressed finding
//! but mark it with `additional_fields.suppressed=true`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionScope {
    NextLine(usize),
    Block { start: usize, end: usize },
}

#[derive(Debug, Default)]
pub struct SuppressionMap {
    /// Per file → list of (check_name, scope).
    /// `check_name == "*"` means all checks.
    pub by_file: BTreeMap<PathBuf, Vec<(String, SuppressionScope)>>,
}

static NEXT_LINE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"//\s*slither-disable-next-line\s+([\w,\-\s]+)").expect("regex")
});
static LINE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"//\s*slither-disable-line\s+([\w,\-\s]+)").expect("regex")
});
static START_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"//\s*slither-disable-start\s+([\w,\-\s]+)").expect("regex"));
static END_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"//\s*slither-disable-end\s+([\w,\-\s]+)").expect("regex"));

impl SuppressionMap {
    pub fn parse_file(&mut self, path: &Path, content: &str) {
        let mut active_blocks: Vec<(String, usize)> = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let lineno = idx + 1;
            if let Some(c) = NEXT_LINE_RE.captures(line) {
                for name in split_names(&c[1]) {
                    self.add(path, &name, SuppressionScope::NextLine(lineno + 1));
                }
            }
            if let Some(c) = LINE_RE.captures(line) {
                for name in split_names(&c[1]) {
                    self.add(path, &name, SuppressionScope::NextLine(lineno));
                }
            }
            if let Some(c) = START_RE.captures(line) {
                for name in split_names(&c[1]) {
                    active_blocks.push((name, lineno));
                }
            }
            if let Some(c) = END_RE.captures(line) {
                for name in split_names(&c[1]) {
                    if let Some(pos) = active_blocks.iter().rposition(|(n, _)| n == &name) {
                        let (n, start) = active_blocks.remove(pos);
                        self.add(
                            path,
                            &n,
                            SuppressionScope::Block {
                                start,
                                end: lineno,
                            },
                        );
                    }
                }
            }
        }
        // Any unclosed blocks are treated as suppressing through EOF.
        let total = content.lines().count();
        for (name, start) in active_blocks {
            self.add(
                path,
                &name,
                SuppressionScope::Block {
                    start,
                    end: total,
                },
            );
        }
    }

    fn add(&mut self, path: &Path, name: &str, scope: SuppressionScope) {
        self.by_file
            .entry(path.to_path_buf())
            .or_default()
            .push((name.to_owned(), scope));
    }

    pub fn is_suppressed(&self, path: &Path, line: usize, check: &str) -> bool {
        let Some(entries) = self.by_file.get(path) else {
            return false;
        };
        entries.iter().any(|(c, scope)| {
            (c == check || c == "*")
                && match *scope {
                    SuppressionScope::NextLine(l) => l == line,
                    SuppressionScope::Block { start, end } => line >= start && line <= end,
                }
        })
    }
}

fn split_names(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(|s| s.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_next_line() {
        let mut m = SuppressionMap::default();
        m.parse_file(
            Path::new("X.sol"),
            "function f() {}\n// slither-disable-next-line reentrancy-eth\nfunction g() {}\n",
        );
        assert!(m.is_suppressed(Path::new("X.sol"), 3, "reentrancy-eth"));
        assert!(!m.is_suppressed(Path::new("X.sol"), 3, "suicidal"));
    }

    #[test]
    fn parses_block() {
        let mut m = SuppressionMap::default();
        m.parse_file(
            Path::new("Y.sol"),
            "// slither-disable-start tx-origin\ncode\nmore\n// slither-disable-end tx-origin\n",
        );
        assert!(m.is_suppressed(Path::new("Y.sol"), 2, "tx-origin"));
        assert!(m.is_suppressed(Path::new("Y.sol"), 3, "tx-origin"));
        assert!(!m.is_suppressed(Path::new("Y.sol"), 5, "tx-origin"));
    }
}
