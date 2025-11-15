//! String and character unescaping.

use std::ops::Range;

/// Errors that can occur during unescaping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnescapeError {
    /// Invalid escape sequence.
    InvalidEscape,
    /// Numeric escape out of range.
    OutOfRange,
    /// Unterminated Unicode escape.
    UnterminatedUnicode,
    /// Invalid Unicode escape.
    InvalidUnicode,
    /// Lone slash at end of string.
    LoneSlash,
}

/// The mode of unescaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnescapeMode {
    /// Unescaping a string literal.
    String,
    /// Unescaping a character literal.
    Char,
}

/// Callback for unescape operations.
pub trait UnescapeCallback {
    /// Called for each character in the output.
    fn on_char(&mut self, c: char);
    /// Called when an error occurs.
    fn on_error(&mut self, error: UnescapeError, range: Range<usize>);
}

/// Unescapes a string or character literal.
///
/// The input should not include the opening and closing quotes.
pub fn unescape(src: &str, mode: UnescapeMode, callback: &mut dyn UnescapeCallback) {
    let mut chars = src.char_indices();
    while let Some((start, c)) = chars.next() {
        match c {
            '\\' => {
                // Escape sequence
                match chars.next() {
                    Some((_, 'n')) => callback.on_char('\n'),
                    Some((_, 'r')) => callback.on_char('\r'),
                    Some((_, 't')) => callback.on_char('\t'),
                    Some((_, '\\')) => callback.on_char('\\'),
                    Some((_, '\'')) => callback.on_char('\''),
                    Some((_, '"')) => callback.on_char('"'),
                    Some((_, '0')) => callback.on_char('\0'),
                    Some((hex_start, 'x')) => {
                        // \xNN hex escape
                        let mut hex_digits = String::new();
                        for _ in 0..2 {
                            if let Some((_, c)) = chars.next() {
                                if c.is_ascii_hexdigit() {
                                    hex_digits.push(c);
                                } else {
                                    callback.on_error(
                                        UnescapeError::InvalidEscape,
                                        start..hex_start + 1 + hex_digits.len(),
                                    );
                                    return;
                                }
                            } else {
                                callback.on_error(
                                    UnescapeError::InvalidEscape,
                                    start..src.len(),
                                );
                                return;
                            }
                        }
                        if let Ok(byte) = u8::from_str_radix(&hex_digits, 16) {
                            if byte <= 0x7F {
                                callback.on_char(byte as char);
                            } else {
                                callback.on_error(UnescapeError::OutOfRange, start..hex_start + 3);
                            }
                        } else {
                            callback.on_error(UnescapeError::InvalidEscape, start..hex_start + 3);
                        }
                    }
                    Some((unicode_start, 'u')) => {
                        // \u{...} Unicode escape
                        if let Some((_, '{')) = chars.next() {
                            let mut hex_digits = String::new();
                            let mut found_close = false;
                            let mut end_pos = unicode_start + 2;

                            while let Some((pos, c)) = chars.next() {
                                end_pos = pos + c.len_utf8();
                                if c == '}' {
                                    found_close = true;
                                    break;
                                } else if c.is_ascii_hexdigit() && hex_digits.len() < 6 {
                                    hex_digits.push(c);
                                } else {
                                    callback.on_error(
                                        UnescapeError::InvalidUnicode,
                                        start..end_pos,
                                    );
                                    return;
                                }
                            }

                            if !found_close {
                                callback.on_error(
                                    UnescapeError::UnterminatedUnicode,
                                    start..src.len(),
                                );
                                return;
                            }

                            if hex_digits.is_empty() {
                                callback.on_error(UnescapeError::InvalidUnicode, start..end_pos);
                                return;
                            }

                            if let Ok(codepoint) = u32::from_str_radix(&hex_digits, 16) {
                                if let Some(c) = char::from_u32(codepoint) {
                                    callback.on_char(c);
                                } else {
                                    callback.on_error(UnescapeError::OutOfRange, start..end_pos);
                                }
                            } else {
                                callback.on_error(UnescapeError::InvalidUnicode, start..end_pos);
                            }
                        } else {
                            callback.on_error(UnescapeError::InvalidEscape, start..unicode_start + 2);
                        }
                    }
                    Some((_, c)) => {
                        // Unknown escape sequence
                        callback.on_error(UnescapeError::InvalidEscape, start..start + 2);
                        callback.on_char(c); // Still emit the character
                    }
                    None => {
                        callback.on_error(UnescapeError::LoneSlash, start..src.len());
                    }
                }
            }
            c => {
                // Regular character
                if mode == UnescapeMode::Char && c == '\n' {
                    // Newlines are not allowed in character literals
                    callback.on_error(UnescapeError::InvalidEscape, start..start + c.len_utf8());
                } else {
                    callback.on_char(c);
                }
            }
        }
    }
}

/// A simple callback that collects characters into a string.
pub struct StringCollector {
    pub result: String,
    pub errors: Vec<(UnescapeError, Range<usize>)>,
}

impl StringCollector {
    pub fn new() -> Self {
        Self {
            result: String::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for StringCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl UnescapeCallback for StringCollector {
    fn on_char(&mut self, c: char) {
        self.result.push(c);
    }

    fn on_error(&mut self, error: UnescapeError, range: Range<usize>) {
        self.errors.push((error, range));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unescape_string(src: &str) -> (String, Vec<UnescapeError>) {
        let mut collector = StringCollector::new();
        unescape(src, UnescapeMode::String, &mut collector);
        (
            collector.result,
            collector.errors.into_iter().map(|(e, _)| e).collect(),
        )
    }

    #[test]
    fn test_simple_string() {
        let (result, errors) = unescape_string("hello");
        assert_eq!(result, "hello");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_escape_sequences() {
        let (result, errors) = unescape_string("\\n\\r\\t\\\\");
        assert_eq!(result, "\n\r\t\\");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_quotes() {
        let (result, errors) = unescape_string("\\'\\\"");
        assert_eq!(result, "'\"");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_hex_escape() {
        let (result, errors) = unescape_string("\\x41\\x42");
        assert_eq!(result, "AB");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unicode_escape() {
        let (result, errors) = unescape_string("\\u{1F600}");
        assert_eq!(result, "😀");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_escape() {
        let (_, errors) = unescape_string("\\q");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], UnescapeError::InvalidEscape);
    }

    #[test]
    fn test_invalid_hex() {
        let (_, errors) = unescape_string("\\xGG");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], UnescapeError::InvalidEscape);
    }

    #[test]
    fn test_unterminated_unicode() {
        let (_, errors) = unescape_string("\\u{1F600");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], UnescapeError::UnterminatedUnicode);
    }

    #[test]
    fn test_lone_slash() {
        let (_, errors) = unescape_string("hello\\");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], UnescapeError::LoneSlash);
    }
}
