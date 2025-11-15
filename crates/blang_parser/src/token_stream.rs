//! Token stream for parser.

use crate::utils::TokenExt;
use blang_lexer::{Lexer, Token, TokenKind};
use blang_span::Span;

/// A token stream that supports lookahead and position tracking.
pub struct TokenStream<'a> {
    /// The lexer producing tokens.
    lexer: Lexer<'a>,
    /// Lookahead buffer for peeking ahead.
    buffer: Vec<Token>,
    /// Current position in the stream.
    pos: usize,
    /// The last consumed token (for error reporting).
    last_token: Option<Token>,
}

impl<'a> TokenStream<'a> {
    /// Create a new token stream from source code.
    pub fn new(source: &'a str) -> Self {
        TokenStream {
            lexer: Lexer::new(source),
            buffer: Vec::new(),
            pos: 0,
            last_token: None,
        }
    }

    /// Peek at the current token without consuming it.
    pub fn peek(&mut self) -> &Token {
        self.lookahead(0)
    }

    /// Peek at a token N positions ahead without consuming.
    pub fn lookahead(&mut self, n: usize) -> &Token {
        // Fill buffer if needed
        while self.buffer.len() <= self.pos + n {
            let token = self.lexer.next_token();
            self.buffer.push(token);
        }

        &self.buffer[self.pos + n]
    }

    /// Get the kind of the current token.
    pub fn peek_kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    /// Check if the current token matches the given kind.
    pub fn at(&mut self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    /// Check if we're at the end of file.
    pub fn at_eof(&mut self) -> bool {
        self.at(TokenKind::Eof)
    }

    /// Consume and return the current token.
    pub fn next(&mut self) -> Token {
        let token = self.peek().clone();
        self.last_token = Some(token.clone());
        self.pos += 1;
        token
    }

    /// Consume the current token if it matches the given kind.
    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.next();
            true
        } else {
            false
        }
    }

    /// Consume and return the current token if it matches the given kind.
    pub fn eat_token(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.next())
        } else {
            None
        }
    }

    /// Expect and consume a token of the given kind, or return an error.
    pub fn expect(&mut self, kind: TokenKind) -> Result<Token, (TokenKind, Span)> {
        if self.at(kind) {
            Ok(self.next())
        } else {
            let found = self.peek();
            Err((found.kind, found.span()))
        }
    }

    /// Get the span of the current token.
    pub fn current_span(&mut self) -> Span {
        self.peek().span()
    }

    /// Get the span of the last consumed token.
    pub fn last_span(&self) -> Span {
        self.last_token
            .as_ref()
            .map(|t| t.span())
            .unwrap_or(Span::from_offsets(0, 0))
    }

    /// Skip whitespace and comments.
    pub fn skip_trivia(&mut self) {
        while matches!(
            self.peek_kind(),
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. }
        ) {
            self.next();
        }
    }

    /// Create a checkpoint for potential backtracking.
    pub fn checkpoint(&self) -> usize {
        self.pos
    }

    /// Restore to a previous checkpoint.
    pub fn restore(&mut self, checkpoint: usize) {
        self.pos = checkpoint;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stream_basic() {
        let source = "let x = 42;";
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.peek_kind(), TokenKind::Let);
        assert!(stream.at(TokenKind::Let));
        assert!(!stream.at_eof());

        let token = stream.next();
        assert_eq!(token.kind, TokenKind::Let);

        assert_eq!(stream.peek_kind(), TokenKind::Ident);
    }

    #[test]
    fn test_eat() {
        let source = "fn foo()";
        let mut stream = TokenStream::new(source);

        assert!(stream.eat(TokenKind::Fn));
        assert_eq!(stream.peek_kind(), TokenKind::Ident);
        assert!(!stream.eat(TokenKind::Let));
        assert_eq!(stream.peek_kind(), TokenKind::Ident);
    }

    #[test]
    fn test_lookahead() {
        let source = "a b c";
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.lookahead(0).kind, TokenKind::Ident);
        assert_eq!(stream.lookahead(1).kind, TokenKind::Ident);
        assert_eq!(stream.lookahead(2).kind, TokenKind::Ident);
        assert_eq!(stream.lookahead(3).kind, TokenKind::Eof);

        // Still at first token
        assert_eq!(stream.peek_kind(), TokenKind::Ident);
    }

    #[test]
    fn test_checkpoint_restore() {
        let source = "a b c";
        let mut stream = TokenStream::new(source);

        let checkpoint = stream.checkpoint();
        assert_eq!(stream.next().kind, TokenKind::Ident);
        assert_eq!(stream.next().kind, TokenKind::Ident);
        assert_eq!(stream.peek_kind(), TokenKind::Ident);

        stream.restore(checkpoint);
        assert_eq!(stream.peek_kind(), TokenKind::Ident);
    }

    #[test]
    fn test_expect() {
        let source = "fn";
        let mut stream = TokenStream::new(source);

        let result = stream.expect(TokenKind::Fn);
        assert!(result.is_ok());

        let result = stream.expect(TokenKind::Let);
        assert!(result.is_err());
    }

    #[test]
    fn test_last_span() {
        let source = "let x";
        let mut stream = TokenStream::new(source);

        stream.next(); // consume 'let'
        let span = stream.last_span();
        assert!(span.len() > 0);
    }
}
