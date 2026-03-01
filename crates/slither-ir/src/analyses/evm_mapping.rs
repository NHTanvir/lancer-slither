//! Source map → EVM PC mapping (for the `evm` printer / `slither-read-storage`).
//!
//! solc emits a compressed source-map string. We decode it line-by-line; each
//! entry is `start:length:file_index:jump_type[:modifier_depth]` with empty
//! fields inheriting the previous entry's value.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub start: i64,
    pub length: i64,
    pub file_id: i32,
    pub jump: char,
    pub modifier_depth: i64,
}

pub fn decode(source_map: &str) -> Vec<SourceMapEntry> {
    let mut out: Vec<SourceMapEntry> = Vec::new();
    let mut last = SourceMapEntry {
        start: 0,
        length: 0,
        file_id: 0,
        jump: '-',
        modifier_depth: 0,
    };
    for chunk in source_map.split(';') {
        let mut fields = chunk.split(':');
        let s = fields.next().unwrap_or("");
        let l = fields.next().unwrap_or("");
        let f = fields.next().unwrap_or("");
        let j = fields.next().unwrap_or("");
        let m = fields.next().unwrap_or("");
        let entry = SourceMapEntry {
            start: if s.is_empty() { last.start } else { s.parse().unwrap_or(last.start) },
            length: if l.is_empty() { last.length } else { l.parse().unwrap_or(last.length) },
            file_id: if f.is_empty() { last.file_id } else { f.parse().unwrap_or(last.file_id) },
            jump: j.chars().next().unwrap_or(last.jump),
            modifier_depth: if m.is_empty() { last.modifier_depth } else { m.parse().unwrap_or(last.modifier_depth) },
        };
        last = entry.clone();
        out.push(entry);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_inherit() {
        let m = decode("100:200:0:o:0;:50:::");
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].start, 100);
        assert_eq!(m[1].length, 50);
        assert_eq!(m[1].start, 100);
    }
}
