//! Position types for source code locations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A byte offset in a source file.
///
/// Byte positions are 0-indexed and represent the number of bytes from the
/// start of the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BytePos(pub u32);

impl BytePos {
    /// Create a new byte position.
    pub const fn new(pos: u32) -> Self {
        BytePos(pos)
    }

    /// Get the position as a u32.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Get the position as a usize.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Add an offset to this position.
    pub const fn offset(self, offset: u32) -> Self {
        BytePos(self.0 + offset)
    }

    /// Subtract an offset from this position.
    pub const fn checked_sub(self, offset: u32) -> Option<Self> {
        match self.0.checked_sub(offset) {
            Some(pos) => Some(BytePos(pos)),
            None => None,
        }
    }

    /// Calculate the distance between two positions.
    pub const fn distance(self, other: BytePos) -> u32 {
        if self.0 >= other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        }
    }
}

impl From<u32> for BytePos {
    fn from(pos: u32) -> Self {
        BytePos(pos)
    }
}

impl From<usize> for BytePos {
    fn from(pos: usize) -> Self {
        BytePos(pos as u32)
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add<u32> for BytePos {
    type Output = BytePos;

    fn add(self, rhs: u32) -> Self::Output {
        BytePos(self.0 + rhs)
    }
}

impl std::ops::Sub<u32> for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: u32) -> Self::Output {
        BytePos(self.0 - rhs)
    }
}

impl std::ops::Sub<BytePos> for BytePos {
    type Output = u32;

    fn sub(self, rhs: BytePos) -> Self::Output {
        self.0 - rhs.0
    }
}

/// A character offset in a source file.
///
/// Character positions are 0-indexed and represent the number of Unicode
/// scalar values from the start of the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CharPos(pub u32);

impl CharPos {
    /// Create a new character position.
    pub const fn new(pos: u32) -> Self {
        CharPos(pos)
    }

    /// Get the position as a u32.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Get the position as a usize.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for CharPos {
    fn from(pos: u32) -> Self {
        CharPos(pos)
    }
}

impl From<usize> for CharPos {
    fn from(pos: usize) -> Self {
        CharPos(pos as u32)
    }
}

impl fmt::Display for CharPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A line and column position in a source file.
///
/// Both line and column numbers are 1-indexed for human-readable output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineCol {
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub col: u32,
}

impl LineCol {
    /// Create a new line/column position.
    pub const fn new(line: u32, col: u32) -> Self {
        LineCol { line, col }
    }

    /// Create a line/column position from 0-indexed values.
    pub const fn from_zero_indexed(line: u32, col: u32) -> Self {
        LineCol {
            line: line + 1,
            col: col + 1,
        }
    }

    /// Get the line number (1-indexed).
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Get the column number (1-indexed).
    pub const fn col(self) -> u32 {
        self.col
    }

    /// Get the line number (0-indexed).
    pub const fn line_zero_indexed(self) -> u32 {
        self.line - 1
    }

    /// Get the column number (0-indexed).
    pub const fn col_zero_indexed(self) -> u32 {
        self.col - 1
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_pos() {
        let pos = BytePos::new(10);
        assert_eq!(pos.as_u32(), 10);
        assert_eq!(pos.as_usize(), 10);

        let offset = pos.offset(5);
        assert_eq!(offset.as_u32(), 15);

        let sub = pos.checked_sub(5);
        assert_eq!(sub, Some(BytePos::new(5)));

        let distance = pos.distance(BytePos::new(20));
        assert_eq!(distance, 10);
    }

    #[test]
    fn test_byte_pos_arithmetic() {
        let pos = BytePos::new(10);
        assert_eq!(pos + 5, BytePos::new(15));
        assert_eq!(pos - 5, BytePos::new(5));
        assert_eq!(pos - BytePos::new(5), 5);
    }

    #[test]
    fn test_line_col() {
        let lc = LineCol::new(5, 10);
        assert_eq!(lc.line(), 5);
        assert_eq!(lc.col(), 10);
        assert_eq!(lc.line_zero_indexed(), 4);
        assert_eq!(lc.col_zero_indexed(), 9);

        let lc2 = LineCol::from_zero_indexed(4, 9);
        assert_eq!(lc2.line(), 5);
        assert_eq!(lc2.col(), 10);
    }

    #[test]
    fn test_display() {
        let pos = BytePos::new(42);
        assert_eq!(pos.to_string(), "42");

        let lc = LineCol::new(5, 10);
        assert_eq!(lc.to_string(), "5:10");
    }
}
