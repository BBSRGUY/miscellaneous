//! Span types for representing ranges in source code.

use crate::pos::BytePos;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A region of source code.
///
/// A span represents a half-open range `[start, end)` of bytes in a source file.
/// Spans are used throughout the compiler for error reporting and source mapping.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// The starting byte position (inclusive).
    pub start: BytePos,
    /// The ending byte position (exclusive).
    pub end: BytePos,
}

impl Span {
    /// A dummy span for use when a span is not available.
    pub const DUMMY: Span = Span {
        start: BytePos(0),
        end: BytePos(0),
    };

    /// Create a new span from start and end positions.
    pub const fn new(start: BytePos, end: BytePos) -> Self {
        Span { start, end }
    }

    /// Create a new span from start and length.
    pub const fn with_len(start: BytePos, len: u32) -> Self {
        Span {
            start,
            end: BytePos(start.0 + len),
        }
    }

    /// Create a span from byte offsets.
    pub const fn from_offsets(start: u32, end: u32) -> Self {
        Span {
            start: BytePos(start),
            end: BytePos(end),
        }
    }

    /// Get the length of this span in bytes.
    pub const fn len(&self) -> u32 {
        self.end.0 - self.start.0
    }

    /// Check if this span is empty (zero length).
    pub const fn is_empty(&self) -> bool {
        self.start.0 == self.end.0
    }

    /// Check if this is a dummy span.
    pub const fn is_dummy(&self) -> bool {
        self.start.0 == 0 && self.end.0 == 0
    }

    /// Get the starting position of this span.
    pub const fn start(&self) -> BytePos {
        self.start
    }

    /// Get the ending position of this span.
    pub const fn end(&self) -> BytePos {
        self.end
    }

    /// Check if this span contains the given position.
    pub const fn contains(&self, pos: BytePos) -> bool {
        self.start.0 <= pos.0 && pos.0 < self.end.0
    }

    /// Check if this span overlaps with another span.
    pub const fn overlaps(&self, other: Span) -> bool {
        self.start.0 < other.end.0 && other.start.0 < self.end.0
    }

    /// Merge two spans into a single span covering both.
    ///
    /// The resulting span will cover the range from the minimum start
    /// position to the maximum end position.
    pub const fn merge(self, other: Span) -> Span {
        let start = if self.start.0 < other.start.0 {
            self.start
        } else {
            other.start
        };
        let end = if self.end.0 > other.end.0 {
            self.end
        } else {
            other.end
        };
        Span { start, end }
    }

    /// Create a span from the start of `self` to the end of `other`.
    pub const fn to(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }

    /// Create a span from the end of `self` to the start of `other`.
    ///
    /// Returns None if there's no gap between the spans.
    pub const fn between(self, other: Span) -> Option<Span> {
        if self.end.0 <= other.start.0 {
            Some(Span {
                start: self.end,
                end: other.start,
            })
        } else {
            None
        }
    }

    /// Shrink this span by the given amount from both sides.
    pub const fn shrink(self, amount: u32) -> Span {
        Span {
            start: BytePos(self.start.0 + amount),
            end: BytePos(self.end.0.saturating_sub(amount)),
        }
    }

    /// Extend this span by the given amount on both sides.
    pub const fn extend(self, amount: u32) -> Span {
        Span {
            start: BytePos(self.start.0.saturating_sub(amount)),
            end: BytePos(self.end.0 + amount),
        }
    }

    /// Get a span covering only the start position.
    pub const fn start_point(self) -> Span {
        Span {
            start: self.start,
            end: self.start,
        }
    }

    /// Get a span covering only the end position.
    pub const fn end_point(self) -> Span {
        Span {
            start: self.end,
            end: self.end,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start.0, self.end.0)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start.0, self.end.0)
    }
}

impl Default for Span {
    fn default() -> Self {
        Span::DUMMY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(BytePos(10), BytePos(20));
        assert_eq!(span.start, BytePos(10));
        assert_eq!(span.end, BytePos(20));
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let span2 = Span::with_len(BytePos(5), 15);
        assert_eq!(span2.start, BytePos(5));
        assert_eq!(span2.end, BytePos(20));

        let span3 = Span::from_offsets(0, 100);
        assert_eq!(span3.start, BytePos(0));
        assert_eq!(span3.end, BytePos(100));
    }

    #[test]
    fn test_span_dummy() {
        let span = Span::DUMMY;
        assert!(span.is_dummy());
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::from_offsets(10, 20);
        assert!(span.contains(BytePos(10)));
        assert!(span.contains(BytePos(15)));
        assert!(!span.contains(BytePos(20))); // end is exclusive
        assert!(!span.contains(BytePos(5)));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::from_offsets(10, 20);
        let span2 = Span::from_offsets(15, 25);
        let span3 = Span::from_offsets(25, 30);

        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));
        assert!(!span1.overlaps(span3));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::from_offsets(10, 20);
        let span2 = Span::from_offsets(25, 30);
        let merged = span1.merge(span2);

        assert_eq!(merged.start, BytePos(10));
        assert_eq!(merged.end, BytePos(30));
    }

    #[test]
    fn test_span_to() {
        let span1 = Span::from_offsets(10, 15);
        let span2 = Span::from_offsets(20, 25);
        let result = span1.to(span2);

        assert_eq!(result.start, BytePos(10));
        assert_eq!(result.end, BytePos(25));
    }

    #[test]
    fn test_span_between() {
        let span1 = Span::from_offsets(10, 15);
        let span2 = Span::from_offsets(20, 25);
        let between = span1.between(span2).unwrap();

        assert_eq!(between.start, BytePos(15));
        assert_eq!(between.end, BytePos(20));

        // Overlapping spans should return None
        let span3 = Span::from_offsets(12, 18);
        assert!(span1.between(span3).is_none());
    }

    #[test]
    fn test_span_shrink_extend() {
        let span = Span::from_offsets(10, 20);
        let shrunk = span.shrink(2);
        assert_eq!(shrunk.start, BytePos(12));
        assert_eq!(shrunk.end, BytePos(18));

        let extended = span.extend(5);
        assert_eq!(extended.start, BytePos(5));
        assert_eq!(extended.end, BytePos(25));
    }

    #[test]
    fn test_span_points() {
        let span = Span::from_offsets(10, 20);
        let start = span.start_point();
        assert_eq!(start.start, BytePos(10));
        assert_eq!(start.end, BytePos(10));

        let end = span.end_point();
        assert_eq!(end.start, BytePos(20));
        assert_eq!(end.end, BytePos(20));
    }

    #[test]
    fn test_display() {
        let span = Span::from_offsets(10, 20);
        assert_eq!(span.to_string(), "10..20");
    }
}
