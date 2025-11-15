//! Global source file manager.

use crate::pos::{BytePos, LineCol};
use crate::source_file::SourceFile;
use crate::span::Span;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// A global registry of all source files.
///
/// The `SourceMap` manages all source files in a compilation session,
/// allowing lookup of file information and conversion of byte positions
/// to line/column numbers.
#[derive(Debug, Clone)]
pub struct SourceMap {
    files: Arc<RwLock<Vec<Arc<SourceFile>>>>,
    next_start_pos: Arc<RwLock<BytePos>>,
}

impl SourceMap {
    /// Create a new empty source map.
    pub fn new() -> Self {
        SourceMap {
            files: Arc::new(RwLock::new(Vec::new())),
            next_start_pos: Arc::new(RwLock::new(BytePos(0))),
        }
    }

    /// Add a new source file to the map.
    ///
    /// Returns an Arc to the newly created SourceFile.
    pub fn add_file(&self, name: PathBuf, src: String) -> Arc<SourceFile> {
        let mut files = self.files.write().unwrap();
        let mut next_pos = self.next_start_pos.write().unwrap();

        let start_pos = *next_pos;
        let file = Arc::new(SourceFile::new(name, src, start_pos));

        // Update next_start_pos to be after this file
        *next_pos = file.end_pos();

        files.push(Arc::clone(&file));
        file
    }

    /// Load a source file from disk and add it to the map.
    pub fn load_file(&self, path: &Path) -> Result<Arc<SourceFile>, std::io::Error> {
        let src = std::fs::read_to_string(path)?;
        Ok(self.add_file(path.to_path_buf(), src))
    }

    /// Get a source file by index.
    pub fn get_file(&self, index: usize) -> Option<Arc<SourceFile>> {
        let files = self.files.read().unwrap();
        files.get(index).cloned()
    }

    /// Find the source file containing the given byte position.
    pub fn lookup_file(&self, pos: BytePos) -> Option<Arc<SourceFile>> {
        let files = self.files.read().unwrap();

        // Binary search for the file containing this position
        files
            .iter()
            .find(|file| file.start_pos <= pos && pos < file.end_pos())
            .cloned()
    }

    /// Find the source file containing the given span.
    pub fn lookup_file_for_span(&self, span: Span) -> Option<Arc<SourceFile>> {
        self.lookup_file(span.start)
    }

    /// Convert a byte position to a line and column.
    pub fn lookup_line_col(&self, pos: BytePos) -> Option<LineCol> {
        let file = self.lookup_file(pos)?;
        file.lookup_line_col(pos)
    }

    /// Get the source text for a span.
    pub fn span_to_snippet(&self, span: Span) -> Option<String> {
        let file = self.lookup_file(span.start)?;
        file.source_text(span.start, span.end)
            .map(|s| s.to_string())
    }

    /// Get the file name for a span.
    pub fn span_to_filename(&self, span: Span) -> Option<String> {
        let file = self.lookup_file(span.start)?;
        Some(file.name_str().to_string())
    }

    /// Get all files in the source map.
    pub fn files(&self) -> Vec<Arc<SourceFile>> {
        let files = self.files.read().unwrap();
        files.clone()
    }

    /// Get the number of files in the source map.
    pub fn file_count(&self) -> usize {
        let files = self.files.read().unwrap();
        files.len()
    }

    /// Check if the source map is empty.
    pub fn is_empty(&self) -> bool {
        self.file_count() == 0
    }

    /// Clear all files from the source map.
    pub fn clear(&self) {
        let mut files = self.files.write().unwrap();
        let mut next_pos = self.next_start_pos.write().unwrap();
        files.clear();
        *next_pos = BytePos(0);
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map_creation() {
        let sm = SourceMap::new();
        assert!(sm.is_empty());
        assert_eq!(sm.file_count(), 0);
    }

    #[test]
    fn test_add_file() {
        let sm = SourceMap::new();

        let file1 = sm.add_file(PathBuf::from("file1.bl"), "abc\ndef".to_string());
        assert_eq!(file1.start_pos, BytePos(0));
        assert_eq!(file1.len(), 7);

        let file2 = sm.add_file(PathBuf::from("file2.bl"), "ghi\njkl".to_string());
        assert_eq!(file2.start_pos, BytePos(7));
        assert_eq!(file2.len(), 7);

        assert_eq!(sm.file_count(), 2);
    }

    #[test]
    fn test_lookup_file() {
        let sm = SourceMap::new();

        let file1 = sm.add_file(PathBuf::from("file1.bl"), "abc".to_string());
        let file2 = sm.add_file(PathBuf::from("file2.bl"), "def".to_string());

        // Lookup positions in first file
        let found = sm.lookup_file(BytePos(0)).unwrap();
        assert_eq!(found.name, file1.name);

        let found = sm.lookup_file(BytePos(2)).unwrap();
        assert_eq!(found.name, file1.name);

        // Lookup positions in second file
        let found = sm.lookup_file(BytePos(3)).unwrap();
        assert_eq!(found.name, file2.name);

        let found = sm.lookup_file(BytePos(5)).unwrap();
        assert_eq!(found.name, file2.name);

        // Out of bounds
        assert!(sm.lookup_file(BytePos(100)).is_none());
    }

    #[test]
    fn test_lookup_line_col() {
        let sm = SourceMap::new();
        sm.add_file(PathBuf::from("test.bl"), "abc\ndef\nghi".to_string());

        let lc = sm.lookup_line_col(BytePos(0)).unwrap();
        assert_eq!(lc.line(), 1);
        assert_eq!(lc.col(), 1);

        let lc = sm.lookup_line_col(BytePos(4)).unwrap();
        assert_eq!(lc.line(), 2);
        assert_eq!(lc.col(), 1);

        let lc = sm.lookup_line_col(BytePos(8)).unwrap();
        assert_eq!(lc.line(), 3);
        assert_eq!(lc.col(), 1);
    }

    #[test]
    fn test_span_to_snippet() {
        let sm = SourceMap::new();
        sm.add_file(PathBuf::from("test.bl"), "hello world".to_string());

        let span = Span::from_offsets(0, 5);
        assert_eq!(sm.span_to_snippet(span), Some("hello".to_string()));

        let span = Span::from_offsets(6, 11);
        assert_eq!(sm.span_to_snippet(span), Some("world".to_string()));
    }

    #[test]
    fn test_span_to_filename() {
        let sm = SourceMap::new();
        sm.add_file(PathBuf::from("test.bl"), "hello".to_string());

        let span = Span::from_offsets(0, 5);
        assert_eq!(sm.span_to_filename(span), Some("test.bl".to_string()));
    }

    #[test]
    fn test_get_file() {
        let sm = SourceMap::new();

        sm.add_file(PathBuf::from("file1.bl"), "abc".to_string());
        sm.add_file(PathBuf::from("file2.bl"), "def".to_string());

        let file = sm.get_file(0).unwrap();
        assert_eq!(file.name_str(), "file1.bl");

        let file = sm.get_file(1).unwrap();
        assert_eq!(file.name_str(), "file2.bl");

        assert!(sm.get_file(2).is_none());
    }

    #[test]
    fn test_clear() {
        let sm = SourceMap::new();
        sm.add_file(PathBuf::from("test.bl"), "hello".to_string());
        assert_eq!(sm.file_count(), 1);

        sm.clear();
        assert!(sm.is_empty());
        assert_eq!(sm.file_count(), 0);
    }

    #[test]
    fn test_multiple_files_line_col() {
        let sm = SourceMap::new();

        // File 1: "abc\n" (4 bytes)
        sm.add_file(PathBuf::from("file1.bl"), "abc\n".to_string());

        // File 2: "def\nghi" (7 bytes, starts at position 4)
        sm.add_file(PathBuf::from("file2.bl"), "def\nghi".to_string());

        // Position 0 in file1
        let lc = sm.lookup_line_col(BytePos(0)).unwrap();
        assert_eq!(lc.line(), 1);
        assert_eq!(lc.col(), 1);

        // Position 4 in file2 (first character of "def")
        let lc = sm.lookup_line_col(BytePos(4)).unwrap();
        assert_eq!(lc.line(), 1);
        assert_eq!(lc.col(), 1);

        // Position 8 in file2 (first character of "ghi")
        let lc = sm.lookup_line_col(BytePos(8)).unwrap();
        assert_eq!(lc.line(), 2);
        assert_eq!(lc.col(), 1);
    }
}
