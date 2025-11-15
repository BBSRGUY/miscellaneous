//! Source file representation.

use crate::pos::{BytePos, LineCol};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Represents a single source file.
///
/// A `SourceFile` contains the source text and metadata needed for
/// error reporting, including line break positions for converting
/// byte positions to line/column numbers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    /// The name or path of the source file.
    pub name: PathBuf,
    /// The source code as a string.
    #[serde(with = "arc_str_serde")]
    pub src: Arc<String>,
    /// Starting byte position of this file in the global source map.
    pub start_pos: BytePos,
    /// Byte positions of each newline character.
    ///
    /// This allows efficient conversion from byte positions to line/column.
    /// Each element represents the byte position immediately after a '\n'.
    pub newlines: Vec<BytePos>,
}

/// Custom serde implementation for Arc<String>
mod arc_str_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(arc: &Arc<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arc.as_str().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Arc::new)
    }
}

impl SourceFile {
    /// Create a new source file.
    pub fn new(name: PathBuf, src: String, start_pos: BytePos) -> Self {
        let newlines = Self::compute_newlines(&src, start_pos);
        SourceFile {
            name,
            src: Arc::new(src),
            start_pos,
            newlines,
        }
    }

    /// Compute the positions of all newline characters in the source.
    fn compute_newlines(src: &str, start_pos: BytePos) -> Vec<BytePos> {
        src.bytes()
            .enumerate()
            .filter_map(|(i, b)| {
                if b == b'\n' {
                    // Position after the newline
                    Some(start_pos + (i as u32 + 1))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the ending byte position of this file.
    pub fn end_pos(&self) -> BytePos {
        self.start_pos + self.src.len() as u32
    }

    /// Get the length of this file in bytes.
    pub fn len(&self) -> usize {
        self.src.len()
    }

    /// Check if this file is empty.
    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }

    /// Get the file name as a string.
    pub fn name_str(&self) -> &str {
        self.name.to_str().unwrap_or("<unnamed>")
    }

    /// Convert a byte position to a line and column.
    ///
    /// Returns None if the position is not within this file.
    pub fn lookup_line_col(&self, pos: BytePos) -> Option<LineCol> {
        if pos < self.start_pos || pos > self.end_pos() {
            return None;
        }

        // Binary search to find the line containing this position
        let line = match self.newlines.binary_search(&pos) {
            Ok(idx) => idx + 1,           // Exact match on newline position
            Err(idx) => idx,              // Position is between newlines
        };

        // Calculate column offset
        let line_start = if line == 0 {
            self.start_pos
        } else {
            self.newlines[line - 1]
        };

        let col = pos.0 - line_start.0;

        Some(LineCol::from_zero_indexed(line as u32, col))
    }

    /// Get the source text for a given line (0-indexed).
    pub fn get_line(&self, line: usize) -> Option<&str> {
        let start = if line == 0 {
            0
        } else {
            let newline_pos = self.newlines.get(line - 1)?;
            (newline_pos.0 - self.start_pos.0) as usize
        };

        let end = if line < self.newlines.len() {
            // Line ends at the newline character (exclusive)
            (self.newlines[line].0 - self.start_pos.0 - 1) as usize
        } else {
            // Last line extends to end of file
            self.src.len()
        };

        if start <= end && end <= self.src.len() {
            Some(&self.src[start..end])
        } else {
            None
        }
    }

    /// Get the total number of lines in this file.
    pub fn line_count(&self) -> usize {
        self.newlines.len() + 1
    }

    /// Get a slice of the source text.
    pub fn source_text(&self, start: BytePos, end: BytePos) -> Option<&str> {
        if start < self.start_pos || end > self.end_pos() {
            return None;
        }

        let start_offset = (start.0 - self.start_pos.0) as usize;
        let end_offset = (end.0 - self.start_pos.0) as usize;

        self.src.get(start_offset..end_offset)
    }
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Arc::ptr_eq(&self.src, &other.src)
    }
}

impl Eq for SourceFile {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_creation() {
        let src = "line1\nline2\nline3".to_string();
        let file = SourceFile::new(PathBuf::from("test.bl"), src.clone(), BytePos(0));

        assert_eq!(file.name_str(), "test.bl");
        assert_eq!(file.len(), src.len());
        assert_eq!(file.start_pos, BytePos(0));
        assert_eq!(file.end_pos(), BytePos(src.len() as u32));
        assert_eq!(file.line_count(), 3);
    }

    #[test]
    fn test_newline_positions() {
        let src = "abc\ndef\nghi".to_string();
        let file = SourceFile::new(PathBuf::from("test.bl"), src, BytePos(0));

        // Newlines are after positions 3 and 7 (after 'c' and 'f')
        assert_eq!(file.newlines.len(), 2);
        assert_eq!(file.newlines[0], BytePos(4)); // After first \n
        assert_eq!(file.newlines[1], BytePos(8)); // After second \n
    }

    #[test]
    fn test_lookup_line_col() {
        let src = "abc\ndef\nghi".to_string();
        let file = SourceFile::new(PathBuf::from("test.bl"), src, BytePos(0));

        // First line
        let lc = file.lookup_line_col(BytePos(0)).unwrap();
        assert_eq!(lc.line(), 1);
        assert_eq!(lc.col(), 1);

        let lc = file.lookup_line_col(BytePos(2)).unwrap();
        assert_eq!(lc.line(), 1);
        assert_eq!(lc.col(), 3);

        // Second line (starts at position 4)
        let lc = file.lookup_line_col(BytePos(4)).unwrap();
        assert_eq!(lc.line(), 2);
        assert_eq!(lc.col(), 1);

        // Third line (starts at position 8)
        let lc = file.lookup_line_col(BytePos(8)).unwrap();
        assert_eq!(lc.line(), 3);
        assert_eq!(lc.col(), 1);
    }

    #[test]
    fn test_get_line() {
        let src = "abc\ndef\nghi".to_string();
        let file = SourceFile::new(PathBuf::from("test.bl"), src, BytePos(0));

        assert_eq!(file.get_line(0), Some("abc"));
        assert_eq!(file.get_line(1), Some("def"));
        assert_eq!(file.get_line(2), Some("ghi"));
        assert_eq!(file.get_line(3), None);
    }

    #[test]
    fn test_source_text() {
        let src = "abc\ndef\nghi".to_string();
        let file = SourceFile::new(PathBuf::from("test.bl"), src, BytePos(0));

        assert_eq!(file.source_text(BytePos(0), BytePos(3)), Some("abc"));
        assert_eq!(file.source_text(BytePos(4), BytePos(7)), Some("def"));
        assert_eq!(file.source_text(BytePos(0), BytePos(11)), Some("abc\ndef\nghi"));

        // Out of bounds
        assert_eq!(file.source_text(BytePos(0), BytePos(100)), None);
    }

    #[test]
    fn test_empty_file() {
        let file = SourceFile::new(PathBuf::from("empty.bl"), String::new(), BytePos(0));
        assert!(file.is_empty());
        assert_eq!(file.line_count(), 1);
        assert_eq!(file.get_line(0), Some(""));
    }
}
