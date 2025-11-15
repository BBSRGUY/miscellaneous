//! Lexer implementation for Blang.

use crate::cursor::{
    is_bin_digit, is_dec_digit, is_hex_digit, is_id_continue, is_id_start, is_oct_digit,
    is_whitespace, Cursor,
};
use crate::token::{str_to_keyword, Base, Token, TokenKind};

/// A lexer that converts source text into a stream of tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// The source text being lexed.
    source: &'a str,
    /// The character cursor.
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source text.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source),
        }
    }

    /// Returns a slice of the source text.
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.source[start..end]
    }

    /// Lexes the next token from the source.
    pub fn next_token(&mut self) -> Token {
        let start = self.cursor.pos();

        let kind = match self.cursor.bump() {
            None => TokenKind::Eof,
            Some(c) => self.lex_after_first_char(c, start),
        };

        let end = self.cursor.pos();
        Token::new(kind, start, end - start)
    }

    fn lex_after_first_char(&mut self, first: char, start: usize) -> TokenKind {
        match first {
            // Whitespace
            c if is_whitespace(c) => {
                self.cursor.bump_while(is_whitespace);
                TokenKind::Whitespace
            }

            // String literals (check before identifiers to catch raw strings)
            '"' => self.lex_string(false),
            'r' if matches!(self.cursor.peek(), Some('"')) => {
                self.cursor.bump(); // consume '"'
                self.lex_string(true)
            }

            // Identifier or keyword
            c if is_id_start(c) => self.lex_identifier_or_keyword(start),

            // Numbers
            '0'..='9' => self.lex_number(first),

            // Character literals
            '\'' => self.lex_char(),

            // Template strings
            '`' => self.lex_template_string(),

            // Comments
            '/' => match self.cursor.peek() {
                Some('/') => self.lex_line_comment(),
                Some('*') => self.lex_block_comment(),
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::SlashEq
                }
                _ => TokenKind::Slash,
            },

            // Operators and punctuation
            '+' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::PlusEq
                }
                _ => TokenKind::Plus,
            },

            '-' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::MinusEq
                }
                Some('>') => {
                    self.cursor.bump();
                    TokenKind::Arrow
                }
                _ => TokenKind::Minus,
            },

            '*' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::StarEq
                }
                _ => TokenKind::Star,
            },

            '%' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::PercentEq
                }
                _ => TokenKind::Percent,
            },

            '=' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::EqEq
                }
                Some('>') => {
                    self.cursor.bump();
                    TokenKind::FatArrow
                }
                _ => TokenKind::Eq,
            },

            '!' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::Ne
                }
                _ => TokenKind::Bang,
            },

            '<' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::Le
                }
                Some('<') => {
                    self.cursor.bump();
                    match self.cursor.peek() {
                        Some('=') => {
                            self.cursor.bump();
                            TokenKind::ShlEq
                        }
                        _ => TokenKind::Shl,
                    }
                }
                _ => TokenKind::Lt,
            },

            '>' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::Ge
                }
                Some('>') => {
                    self.cursor.bump();
                    match self.cursor.peek() {
                        Some('=') => {
                            self.cursor.bump();
                            TokenKind::ShrEq
                        }
                        _ => TokenKind::Shr,
                    }
                }
                _ => TokenKind::Gt,
            },

            '&' => match self.cursor.peek() {
                Some('&') => {
                    self.cursor.bump();
                    TokenKind::AndAnd
                }
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::AndEq
                }
                _ => TokenKind::And,
            },

            '|' => match self.cursor.peek() {
                Some('|') => {
                    self.cursor.bump();
                    TokenKind::OrOr
                }
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::OrEq
                }
                _ => TokenKind::Or,
            },

            '^' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenKind::CaretEq
                }
                _ => TokenKind::Caret,
            },

            '~' => TokenKind::Tilde,

            '.' => match self.cursor.peek() {
                Some('.') => {
                    self.cursor.bump();
                    match self.cursor.peek() {
                        Some('=') => {
                            self.cursor.bump();
                            TokenKind::DotDotEq
                        }
                        _ => TokenKind::DotDot,
                    }
                }
                _ => TokenKind::Dot,
            },

            ':' => match self.cursor.peek() {
                Some(':') => {
                    self.cursor.bump();
                    TokenKind::ColonColon
                }
                _ => TokenKind::Colon,
            },

            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            ';' => TokenKind::Semi,
            ',' => TokenKind::Comma,
            '@' => TokenKind::At,
            '#' => TokenKind::Hash,
            '$' => TokenKind::Dollar,

            _ => TokenKind::Unknown,
        }
    }

    fn lex_identifier_or_keyword(&mut self, start: usize) -> TokenKind {
        self.cursor.bump_while(is_id_continue);
        let end = self.cursor.pos();
        let text = self.slice(start, end);

        str_to_keyword(text).unwrap_or(TokenKind::Ident)
    }

    fn lex_number(&mut self, first: char) -> TokenKind {
        if first == '0' {
            match self.cursor.peek() {
                Some('b') => {
                    self.cursor.bump();
                    return self.lex_integer(Base::Binary);
                }
                Some('o') => {
                    self.cursor.bump();
                    return self.lex_integer(Base::Octal);
                }
                Some('x') => {
                    self.cursor.bump();
                    return self.lex_integer(Base::Hexadecimal);
                }
                _ => {}
            }
        }

        // Decimal integer or float
        self.cursor.bump_while(|c| is_dec_digit(c) || c == '_');

        match self.cursor.peek() {
            Some('.') if matches!(self.cursor.peek_second(), Some('0'..='9')) => {
                // Float with decimal point
                self.cursor.bump(); // consume '.'
                self.cursor.bump_while(|c| is_dec_digit(c) || c == '_');
                self.lex_float_exponent();
                TokenKind::Float { empty: false }
            }
            Some('e') | Some('E') => {
                // Float with exponent
                self.lex_float_exponent();
                TokenKind::Float { empty: false }
            }
            _ => TokenKind::Integer {
                base: Base::Decimal,
                empty: false,
            },
        }
    }

    fn lex_integer(&mut self, base: Base) -> TokenKind {
        let start_pos = self.cursor.pos();

        let digit_check: fn(char) -> bool = match base {
            Base::Binary => is_bin_digit,
            Base::Octal => is_oct_digit,
            Base::Decimal => is_dec_digit,
            Base::Hexadecimal => is_hex_digit,
        };

        self.cursor.bump_while(|c| digit_check(c) || c == '_');

        let empty = self.cursor.pos() == start_pos;
        TokenKind::Integer { base, empty }
    }

    fn lex_float_exponent(&mut self) {
        if matches!(self.cursor.peek(), Some('e') | Some('E')) {
            self.cursor.bump();
            if matches!(self.cursor.peek(), Some('+') | Some('-')) {
                self.cursor.bump();
            }
            self.cursor.bump_while(|c| is_dec_digit(c) || c == '_');
        }
    }

    fn lex_string(&mut self, raw: bool) -> TokenKind {
        let mut terminated = false;

        while let Some(c) = self.cursor.bump() {
            match c {
                '"' => {
                    terminated = true;
                    break;
                }
                '\\' if !raw => {
                    // Consume escape sequence
                    self.cursor.bump();
                }
                _ => {}
            }
        }

        TokenKind::String { raw, terminated }
    }

    fn lex_char(&mut self) -> TokenKind {
        let mut terminated = false;

        while let Some(c) = self.cursor.bump() {
            match c {
                '\'' => {
                    terminated = true;
                    break;
                }
                '\\' => {
                    // Consume escape sequence
                    self.cursor.bump();
                }
                '\n' => {
                    // Newlines not allowed in char literals
                    break;
                }
                _ => {}
            }
        }

        TokenKind::Char { terminated }
    }

    fn lex_template_string(&mut self) -> TokenKind {
        let mut terminated = false;

        while let Some(c) = self.cursor.bump() {
            match c {
                '`' => {
                    terminated = true;
                    break;
                }
                '\\' => {
                    // Consume escape sequence
                    self.cursor.bump();
                }
                '$' if matches!(self.cursor.peek(), Some('{')) => {
                    // Skip template interpolation - we just tokenize the whole thing
                    self.cursor.bump(); // consume '{'
                                        // Find matching '}'
                    let mut depth = 1;
                    while depth > 0 {
                        match self.cursor.bump() {
                            Some('{') => depth += 1,
                            Some('}') => depth -= 1,
                            None => break,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        TokenKind::TemplateString { terminated }
    }

    fn lex_line_comment(&mut self) -> TokenKind {
        self.cursor.bump(); // consume second '/'
        self.cursor.bump_while(|c| c != '\n');
        TokenKind::LineComment
    }

    fn lex_block_comment(&mut self) -> TokenKind {
        self.cursor.bump(); // consume '*'

        let mut depth = 1;
        while depth > 0 {
            match self.cursor.bump() {
                Some('*') if matches!(self.cursor.peek(), Some('/')) => {
                    self.cursor.bump(); // consume '/'
                    depth -= 1;
                }
                Some('/') if matches!(self.cursor.peek(), Some('*')) => {
                    self.cursor.bump(); // consume '*'
                    depth += 1;
                }
                None => return TokenKind::BlockComment { terminated: false },
                _ => {}
            }
        }

        TokenKind::BlockComment { terminated: true }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_single(source: &str) -> TokenKind {
        let mut lexer = Lexer::new(source);
        lexer.next_token().kind
    }

    fn lex_all(source: &str) -> Vec<TokenKind> {
        Lexer::new(source).map(|t| t.kind).collect()
    }

    #[test]
    fn test_keywords() {
        assert_eq!(lex_single("fn"), TokenKind::Fn);
        assert_eq!(lex_single("let"), TokenKind::Let);
        assert_eq!(lex_single("if"), TokenKind::If);
        assert_eq!(lex_single("component"), TokenKind::Component);
        assert_eq!(lex_single("script"), TokenKind::Script);
        assert_eq!(lex_single("module"), TokenKind::Module);
        assert_eq!(lex_single("view"), TokenKind::View);
        assert_eq!(lex_single("style"), TokenKind::Style);
        assert_eq!(lex_single("state"), TokenKind::State);
        assert_eq!(lex_single("signal"), TokenKind::Signal);
        assert_eq!(lex_single("job"), TokenKind::Job);
        assert_eq!(lex_single("step"), TokenKind::Step);
        assert_eq!(lex_single("unsafe"), TokenKind::Unsafe);
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(lex_single("foo"), TokenKind::Ident);
        assert_eq!(lex_single("_bar"), TokenKind::Ident);
        assert_eq!(lex_single("baz123"), TokenKind::Ident);
        assert_eq!(lex_single("camelCase"), TokenKind::Ident);
    }

    #[test]
    fn test_integers() {
        assert_eq!(
            lex_single("42"),
            TokenKind::Integer {
                base: Base::Decimal,
                empty: false
            }
        );
        assert_eq!(
            lex_single("0xFF"),
            TokenKind::Integer {
                base: Base::Hexadecimal,
                empty: false
            }
        );
        assert_eq!(
            lex_single("0o77"),
            TokenKind::Integer {
                base: Base::Octal,
                empty: false
            }
        );
        assert_eq!(
            lex_single("0b1010"),
            TokenKind::Integer {
                base: Base::Binary,
                empty: false
            }
        );
        assert_eq!(
            lex_single("1_000_000"),
            TokenKind::Integer {
                base: Base::Decimal,
                empty: false
            }
        );
    }

    #[test]
    fn test_floats() {
        assert_eq!(lex_single("3.14"), TokenKind::Float { empty: false });
        assert_eq!(lex_single("1e10"), TokenKind::Float { empty: false });
        assert_eq!(lex_single("1.5e-5"), TokenKind::Float { empty: false });
        assert_eq!(lex_single("2.5E+3"), TokenKind::Float { empty: false });
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            lex_single(r#""hello""#),
            TokenKind::String {
                raw: false,
                terminated: true
            }
        );
        assert_eq!(
            lex_single(r#"r"raw string""#),
            TokenKind::String {
                raw: true,
                terminated: true
            }
        );
        assert_eq!(
            lex_single(r#""unterminated"#),
            TokenKind::String {
                raw: false,
                terminated: false
            }
        );
    }

    #[test]
    fn test_chars() {
        assert_eq!(lex_single("'a'"), TokenKind::Char { terminated: true });
        assert_eq!(lex_single("'\\n'"), TokenKind::Char { terminated: true });
        assert_eq!(lex_single("'unterminated"), TokenKind::Char { terminated: false });
    }

    #[test]
    fn test_template_strings() {
        assert_eq!(
            lex_single("`hello`"),
            TokenKind::TemplateString { terminated: true }
        );
        assert_eq!(
            lex_single("`hello ${name}`"),
            TokenKind::TemplateString { terminated: true }
        );
        assert_eq!(
            lex_single("`unterminated"),
            TokenKind::TemplateString { terminated: false }
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(lex_single("+"), TokenKind::Plus);
        assert_eq!(lex_single("-"), TokenKind::Minus);
        assert_eq!(lex_single("*"), TokenKind::Star);
        assert_eq!(lex_single("/"), TokenKind::Slash);
        assert_eq!(lex_single("%"), TokenKind::Percent);
        assert_eq!(lex_single("=="), TokenKind::EqEq);
        assert_eq!(lex_single("!="), TokenKind::Ne);
        assert_eq!(lex_single("<"), TokenKind::Lt);
        assert_eq!(lex_single("<="), TokenKind::Le);
        assert_eq!(lex_single(">"), TokenKind::Gt);
        assert_eq!(lex_single(">="), TokenKind::Ge);
        assert_eq!(lex_single("&&"), TokenKind::AndAnd);
        assert_eq!(lex_single("||"), TokenKind::OrOr);
        assert_eq!(lex_single("!"), TokenKind::Bang);
        assert_eq!(lex_single("&"), TokenKind::And);
        assert_eq!(lex_single("|"), TokenKind::Or);
        assert_eq!(lex_single("^"), TokenKind::Caret);
        assert_eq!(lex_single("~"), TokenKind::Tilde);
        assert_eq!(lex_single("<<"), TokenKind::Shl);
        assert_eq!(lex_single(">>"), TokenKind::Shr);
    }

    #[test]
    fn test_assignment_operators() {
        assert_eq!(lex_single("="), TokenKind::Eq);
        assert_eq!(lex_single("+="), TokenKind::PlusEq);
        assert_eq!(lex_single("-="), TokenKind::MinusEq);
        assert_eq!(lex_single("*="), TokenKind::StarEq);
        assert_eq!(lex_single("/="), TokenKind::SlashEq);
        assert_eq!(lex_single("%="), TokenKind::PercentEq);
        assert_eq!(lex_single("&="), TokenKind::AndEq);
        assert_eq!(lex_single("|="), TokenKind::OrEq);
        assert_eq!(lex_single("^="), TokenKind::CaretEq);
        assert_eq!(lex_single("<<="), TokenKind::ShlEq);
        assert_eq!(lex_single(">>="), TokenKind::ShrEq);
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(lex_single("("), TokenKind::OpenParen);
        assert_eq!(lex_single(")"), TokenKind::CloseParen);
        assert_eq!(lex_single("["), TokenKind::OpenBracket);
        assert_eq!(lex_single("]"), TokenKind::CloseBracket);
        assert_eq!(lex_single("{"), TokenKind::OpenBrace);
        assert_eq!(lex_single("}"), TokenKind::CloseBrace);
        assert_eq!(lex_single(";"), TokenKind::Semi);
        assert_eq!(lex_single(":"), TokenKind::Colon);
        assert_eq!(lex_single(","), TokenKind::Comma);
        assert_eq!(lex_single("."), TokenKind::Dot);
        assert_eq!(lex_single(".."), TokenKind::DotDot);
        assert_eq!(lex_single("..="), TokenKind::DotDotEq);
        assert_eq!(lex_single("->"), TokenKind::Arrow);
        assert_eq!(lex_single("=>"), TokenKind::FatArrow);
        assert_eq!(lex_single("::"), TokenKind::ColonColon);
        assert_eq!(lex_single("@"), TokenKind::At);
        assert_eq!(lex_single("#"), TokenKind::Hash);
        assert_eq!(lex_single("$"), TokenKind::Dollar);
    }

    #[test]
    fn test_comments() {
        assert_eq!(lex_single("// comment"), TokenKind::LineComment);
        assert_eq!(
            lex_single("/* comment */"),
            TokenKind::BlockComment { terminated: true }
        );
        assert_eq!(
            lex_single("/* unterminated"),
            TokenKind::BlockComment { terminated: false }
        );
    }

    #[test]
    fn test_nested_block_comments() {
        assert_eq!(
            lex_single("/* outer /* inner */ outer */"),
            TokenKind::BlockComment { terminated: true }
        );
    }

    #[test]
    fn test_simple_expression() {
        let tokens = lex_all("let x = 42;");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Let,
                TokenKind::Whitespace,
                TokenKind::Ident,
                TokenKind::Whitespace,
                TokenKind::Eq,
                TokenKind::Whitespace,
                TokenKind::Integer {
                    base: Base::Decimal,
                    empty: false
                },
                TokenKind::Semi,
            ]
        );
    }

    #[test]
    fn test_function_declaration() {
        let tokens = lex_all("fn add(a: i32, b: i32) -> i32 { a + b }");
        assert!(tokens.contains(&TokenKind::Fn));
        assert!(tokens.contains(&TokenKind::Ident));
        assert!(tokens.contains(&TokenKind::OpenParen));
        assert!(tokens.contains(&TokenKind::CloseParen));
        assert!(tokens.contains(&TokenKind::Arrow));
        assert!(tokens.contains(&TokenKind::OpenBrace));
        assert!(tokens.contains(&TokenKind::CloseBrace));
    }
}
