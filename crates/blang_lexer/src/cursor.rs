//! Character cursor for the lexer.

use std::str::Chars;

/// A cursor over source text that allows peeking and consuming characters.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    /// The remaining characters to lex.
    chars: Chars<'a>,
    /// The initial length of the source (for calculating positions).
    initial_len: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor over the given source.
    pub fn new(source: &'a str) -> Self {
        Self {
            initial_len: source.len(),
            chars: source.chars(),
        }
    }

    /// Returns the current byte position.
    pub fn pos(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }

    /// Peeks at the next character without consuming it.
    pub fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// Peeks at the second character without consuming anything.
    pub fn peek_second(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next()
    }

    /// Consumes and returns the next character.
    pub fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    /// Consumes characters while the predicate returns `true`.
    pub fn bump_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while let Some(c) = self.peek() {
            if predicate(c) {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Returns `true` if there are no more characters to consume.
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns the remaining source text.
    pub fn remaining(&self) -> &'a str {
        self.chars.as_str()
    }
}

/// Returns `true` if the character is a valid identifier start character.
pub fn is_id_start(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_start(c) || c == '_'
}

/// Returns `true` if the character is a valid identifier continuation character.
pub fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

/// Returns `true` if the character is a decimal digit.
pub fn is_dec_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Returns `true` if the character is a binary digit.
pub fn is_bin_digit(c: char) -> bool {
    matches!(c, '0' | '1')
}

/// Returns `true` if the character is an octal digit.
pub fn is_oct_digit(c: char) -> bool {
    matches!(c, '0'..='7')
}

/// Returns `true` if the character is a hexadecimal digit.
pub fn is_hex_digit(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F')
}

/// Returns `true` if the character is whitespace.
pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '\x0B' // vertical tab
            | '\x0C' // form feed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_peek() {
        let cursor = Cursor::new("abc");
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.peek(), Some('a')); // Peek doesn't consume
    }

    #[test]
    fn test_cursor_bump() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.bump(), Some('a'));
        assert_eq!(cursor.bump(), Some('b'));
        assert_eq!(cursor.bump(), Some('c'));
        assert_eq!(cursor.bump(), None);
    }

    #[test]
    fn test_cursor_bump_while() {
        let mut cursor = Cursor::new("aaa123");
        cursor.bump_while(|c| c == 'a');
        assert_eq!(cursor.peek(), Some('1'));
    }

    #[test]
    fn test_cursor_peek_second() {
        let cursor = Cursor::new("abc");
        assert_eq!(cursor.peek_second(), Some('b'));
    }

    #[test]
    fn test_is_id_start() {
        assert!(is_id_start('a'));
        assert!(is_id_start('Z'));
        assert!(is_id_start('_'));
        assert!(!is_id_start('0'));
        assert!(!is_id_start(' '));
    }

    #[test]
    fn test_is_id_continue() {
        assert!(is_id_continue('a'));
        assert!(is_id_continue('0'));
        assert!(is_id_continue('_'));
        assert!(!is_id_continue(' '));
    }
}
