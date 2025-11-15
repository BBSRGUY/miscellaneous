//! Token types and definitions for the Blang lexer.

use std::fmt;

/// A token with its kind, position, and length.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// The byte offset from the start of the source.
    pub start: usize,
    /// The length of the token in bytes.
    pub len: usize,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, start: usize, len: usize) -> Self {
        Self { kind, start, len }
    }

    /// Returns the end position of the token.
    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

/// The kind of a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    /// Integer literal (e.g., `42`, `0xFF`, `0b1010`)
    Integer {
        /// The base of the integer (2, 8, 10, or 16)
        base: Base,
        /// Whether the literal is empty (e.g., `0x` without digits)
        empty: bool,
    },
    /// Floating-point literal (e.g., `3.14`, `1e-5`)
    Float {
        /// Whether the literal is empty or malformed
        empty: bool,
    },
    /// String literal (e.g., `"hello"`, `r"raw"`)
    String {
        /// Whether this is a raw string
        raw: bool,
        /// Whether the string is terminated
        terminated: bool,
    },
    /// Character literal (e.g., `'a'`, `'\n'`)
    Char {
        /// Whether the character is terminated
        terminated: bool,
    },
    /// Template string literal (e.g., `` `Hello ${name}` ``)
    TemplateString {
        /// Whether the string is terminated
        terminated: bool,
    },

    // Keywords
    /// `as`
    As,
    /// `async`
    Async,
    /// `await`
    Await,
    /// `break`
    Break,
    /// `case`
    Case,
    /// `catch`
    Catch,
    /// `component`
    Component,
    /// `computed`
    Computed,
    /// `const`
    Const,
    /// `continue`
    Continue,
    /// `default`
    Default,
    /// `defer`
    Defer,
    /// `depends_on`
    DependsOn,
    /// `do`
    Do,
    /// `else`
    Else,
    /// `effect`
    Effect,
    /// `enum`
    Enum,
    /// `export`
    Export,
    /// `false`
    False,
    /// `fn`
    Fn,
    /// `for`
    For,
    /// `if`
    If,
    /// `impl`
    Impl,
    /// `import`
    Import,
    /// `in`
    In,
    /// `interface`
    Interface,
    /// `job`
    Job,
    /// `let`
    Let,
    /// `loop`
    Loop,
    /// `match`
    Match,
    /// `memo`
    Memo,
    /// `module`
    Module,
    /// `mut`
    Mut,
    /// `null`
    Null,
    /// `on_mount`
    OnMount,
    /// `on_update`
    OnUpdate,
    /// `on_unmount`
    OnUnmount,
    /// `parallel`
    Parallel,
    /// `props`
    Props,
    /// `pub`
    Pub,
    /// `return`
    Return,
    /// `script`
    Script,
    /// `self`
    SelfKw,
    /// `signal`
    Signal,
    /// `state`
    State,
    /// `step`
    Step,
    /// `struct`
    Struct,
    /// `style`
    Style,
    /// `super`
    Super,
    /// `trait`
    Trait,
    /// `true`
    True,
    /// `type`
    Type,
    /// `typeof`
    Typeof,
    /// `unsafe`
    Unsafe,
    /// `use`
    Use,
    /// `view`
    View,
    /// `while`
    While,
    /// `yield`
    Yield,

    // Contextual keywords (can be identifiers in some contexts)
    /// `actor`
    Actor,
    /// `channel`
    Channel,
    /// `receiver`
    Receiver,
    /// `send`
    Send,
    /// `get`
    Get,
    /// `set`
    Set,

    // Identifier
    /// Identifier (e.g., `foo`, `bar_baz`)
    Ident,

    // Operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Bang,
    /// `&`
    And,
    /// `|`
    Or,
    /// `^`
    Caret,
    /// `~`
    Tilde,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `=`
    Eq,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,
    /// `%=`
    PercentEq,
    /// `&=`
    AndEq,
    /// `|=`
    OrEq,
    /// `^=`
    CaretEq,
    /// `<<=`
    ShlEq,
    /// `>>=`
    ShrEq,

    // Punctuation
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `;`
    Semi,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `..=`
    DotDotEq,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `::`
    ColonColon,
    /// `@`
    At,
    /// `#`
    Hash,
    /// `$`
    Dollar,

    // Special
    /// End of file
    Eof,
    /// Unknown character
    Unknown,
    /// Whitespace (only produced if requested)
    Whitespace,
    /// Line comment
    LineComment,
    /// Block comment
    BlockComment {
        /// Whether the comment is terminated
        terminated: bool,
    },
}

/// The base of an integer literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    /// Binary (0b)
    Binary = 2,
    /// Octal (0o)
    Octal = 8,
    /// Decimal
    Decimal = 10,
    /// Hexadecimal (0x)
    Hexadecimal = 16,
}

impl TokenKind {
    /// Returns `true` if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::As
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Component
                | TokenKind::Computed
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Default
                | TokenKind::Defer
                | TokenKind::DependsOn
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Effect
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::False
                | TokenKind::Fn
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Impl
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Interface
                | TokenKind::Job
                | TokenKind::Let
                | TokenKind::Loop
                | TokenKind::Match
                | TokenKind::Memo
                | TokenKind::Module
                | TokenKind::Mut
                | TokenKind::Null
                | TokenKind::OnMount
                | TokenKind::OnUpdate
                | TokenKind::OnUnmount
                | TokenKind::Parallel
                | TokenKind::Props
                | TokenKind::Pub
                | TokenKind::Return
                | TokenKind::Script
                | TokenKind::SelfKw
                | TokenKind::Signal
                | TokenKind::State
                | TokenKind::Step
                | TokenKind::Struct
                | TokenKind::Style
                | TokenKind::Super
                | TokenKind::Trait
                | TokenKind::True
                | TokenKind::Type
                | TokenKind::Typeof
                | TokenKind::Unsafe
                | TokenKind::Use
                | TokenKind::View
                | TokenKind::While
                | TokenKind::Yield
        )
    }

    /// Returns `true` if this token is trivia (whitespace or comment).
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. }
        )
    }

    /// Returns `true` if this token is an error token.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            TokenKind::Unknown
                | TokenKind::String { terminated: false, .. }
                | TokenKind::Char { terminated: false }
                | TokenKind::TemplateString { terminated: false }
                | TokenKind::BlockComment { terminated: false }
                | TokenKind::Integer { empty: true, .. }
                | TokenKind::Float { empty: true }
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Integer { .. } => write!(f, "integer"),
            TokenKind::Float { .. } => write!(f, "float"),
            TokenKind::String { .. } => write!(f, "string"),
            TokenKind::Char { .. } => write!(f, "char"),
            TokenKind::TemplateString { .. } => write!(f, "template string"),
            TokenKind::As => write!(f, "`as`"),
            TokenKind::Async => write!(f, "`async`"),
            TokenKind::Await => write!(f, "`await`"),
            TokenKind::Break => write!(f, "`break`"),
            TokenKind::Case => write!(f, "`case`"),
            TokenKind::Catch => write!(f, "`catch`"),
            TokenKind::Component => write!(f, "`component`"),
            TokenKind::Computed => write!(f, "`computed`"),
            TokenKind::Const => write!(f, "`const`"),
            TokenKind::Continue => write!(f, "`continue`"),
            TokenKind::Default => write!(f, "`default`"),
            TokenKind::Defer => write!(f, "`defer`"),
            TokenKind::DependsOn => write!(f, "`depends_on`"),
            TokenKind::Do => write!(f, "`do`"),
            TokenKind::Else => write!(f, "`else`"),
            TokenKind::Effect => write!(f, "`effect`"),
            TokenKind::Enum => write!(f, "`enum`"),
            TokenKind::Export => write!(f, "`export`"),
            TokenKind::False => write!(f, "`false`"),
            TokenKind::Fn => write!(f, "`fn`"),
            TokenKind::For => write!(f, "`for`"),
            TokenKind::If => write!(f, "`if`"),
            TokenKind::Impl => write!(f, "`impl`"),
            TokenKind::Import => write!(f, "`import`"),
            TokenKind::In => write!(f, "`in`"),
            TokenKind::Interface => write!(f, "`interface`"),
            TokenKind::Job => write!(f, "`job`"),
            TokenKind::Let => write!(f, "`let`"),
            TokenKind::Loop => write!(f, "`loop`"),
            TokenKind::Match => write!(f, "`match`"),
            TokenKind::Memo => write!(f, "`memo`"),
            TokenKind::Module => write!(f, "`module`"),
            TokenKind::Mut => write!(f, "`mut`"),
            TokenKind::Null => write!(f, "`null`"),
            TokenKind::OnMount => write!(f, "`on_mount`"),
            TokenKind::OnUpdate => write!(f, "`on_update`"),
            TokenKind::OnUnmount => write!(f, "`on_unmount`"),
            TokenKind::Parallel => write!(f, "`parallel`"),
            TokenKind::Props => write!(f, "`props`"),
            TokenKind::Pub => write!(f, "`pub`"),
            TokenKind::Return => write!(f, "`return`"),
            TokenKind::Script => write!(f, "`script`"),
            TokenKind::SelfKw => write!(f, "`self`"),
            TokenKind::Signal => write!(f, "`signal`"),
            TokenKind::State => write!(f, "`state`"),
            TokenKind::Step => write!(f, "`step`"),
            TokenKind::Struct => write!(f, "`struct`"),
            TokenKind::Style => write!(f, "`style`"),
            TokenKind::Super => write!(f, "`super`"),
            TokenKind::Trait => write!(f, "`trait`"),
            TokenKind::True => write!(f, "`true`"),
            TokenKind::Type => write!(f, "`type`"),
            TokenKind::Typeof => write!(f, "`typeof`"),
            TokenKind::Unsafe => write!(f, "`unsafe`"),
            TokenKind::Use => write!(f, "`use`"),
            TokenKind::View => write!(f, "`view`"),
            TokenKind::While => write!(f, "`while`"),
            TokenKind::Yield => write!(f, "`yield`"),
            TokenKind::Actor => write!(f, "`actor`"),
            TokenKind::Channel => write!(f, "`channel`"),
            TokenKind::Receiver => write!(f, "`receiver`"),
            TokenKind::Send => write!(f, "`send`"),
            TokenKind::Get => write!(f, "`get`"),
            TokenKind::Set => write!(f, "`set`"),
            TokenKind::Ident => write!(f, "identifier"),
            TokenKind::Plus => write!(f, "`+`"),
            TokenKind::Minus => write!(f, "`-`"),
            TokenKind::Star => write!(f, "`*`"),
            TokenKind::Slash => write!(f, "`/`"),
            TokenKind::Percent => write!(f, "`%`"),
            TokenKind::EqEq => write!(f, "`==`"),
            TokenKind::Ne => write!(f, "`!=`"),
            TokenKind::Lt => write!(f, "`<`"),
            TokenKind::Le => write!(f, "`<=`"),
            TokenKind::Gt => write!(f, "`>`"),
            TokenKind::Ge => write!(f, "`>=`"),
            TokenKind::AndAnd => write!(f, "`&&`"),
            TokenKind::OrOr => write!(f, "`||`"),
            TokenKind::Bang => write!(f, "`!`"),
            TokenKind::And => write!(f, "`&`"),
            TokenKind::Or => write!(f, "`|`"),
            TokenKind::Caret => write!(f, "`^`"),
            TokenKind::Tilde => write!(f, "`~`"),
            TokenKind::Shl => write!(f, "`<<`"),
            TokenKind::Shr => write!(f, "`>>`"),
            TokenKind::Eq => write!(f, "`=`"),
            TokenKind::PlusEq => write!(f, "`+=`"),
            TokenKind::MinusEq => write!(f, "`-=`"),
            TokenKind::StarEq => write!(f, "`*=`"),
            TokenKind::SlashEq => write!(f, "`/=`"),
            TokenKind::PercentEq => write!(f, "`%=`"),
            TokenKind::AndEq => write!(f, "`&=`"),
            TokenKind::OrEq => write!(f, "`|=`"),
            TokenKind::CaretEq => write!(f, "`^=`"),
            TokenKind::ShlEq => write!(f, "`<<=`"),
            TokenKind::ShrEq => write!(f, "`>>=`"),
            TokenKind::OpenParen => write!(f, "`(`"),
            TokenKind::CloseParen => write!(f, "`)`"),
            TokenKind::OpenBracket => write!(f, "`[`"),
            TokenKind::CloseBracket => write!(f, "`]`"),
            TokenKind::OpenBrace => write!(f, "`{{`"),
            TokenKind::CloseBrace => write!(f, "`}}`"),
            TokenKind::Semi => write!(f, "`;`"),
            TokenKind::Colon => write!(f, "`:`"),
            TokenKind::Comma => write!(f, "`,`"),
            TokenKind::Dot => write!(f, "`.`"),
            TokenKind::DotDot => write!(f, "`..`"),
            TokenKind::DotDotEq => write!(f, "`..=`"),
            TokenKind::Arrow => write!(f, "`->`"),
            TokenKind::FatArrow => write!(f, "`=>`"),
            TokenKind::ColonColon => write!(f, "`::`"),
            TokenKind::At => write!(f, "`@`"),
            TokenKind::Hash => write!(f, "`#`"),
            TokenKind::Dollar => write!(f, "`$`"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::Unknown => write!(f, "unknown character"),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::LineComment => write!(f, "line comment"),
            TokenKind::BlockComment { .. } => write!(f, "block comment"),
        }
    }
}

/// Converts a string slice to a keyword token kind, if it is a keyword.
pub fn str_to_keyword(s: &str) -> Option<TokenKind> {
    let kind = match s {
        "as" => TokenKind::As,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "break" => TokenKind::Break,
        "case" => TokenKind::Case,
        "catch" => TokenKind::Catch,
        "component" => TokenKind::Component,
        "computed" => TokenKind::Computed,
        "const" => TokenKind::Const,
        "continue" => TokenKind::Continue,
        "default" => TokenKind::Default,
        "defer" => TokenKind::Defer,
        "depends_on" => TokenKind::DependsOn,
        "do" => TokenKind::Do,
        "else" => TokenKind::Else,
        "effect" => TokenKind::Effect,
        "enum" => TokenKind::Enum,
        "export" => TokenKind::Export,
        "false" => TokenKind::False,
        "fn" => TokenKind::Fn,
        "for" => TokenKind::For,
        "if" => TokenKind::If,
        "impl" => TokenKind::Impl,
        "import" => TokenKind::Import,
        "in" => TokenKind::In,
        "interface" => TokenKind::Interface,
        "job" => TokenKind::Job,
        "let" => TokenKind::Let,
        "loop" => TokenKind::Loop,
        "match" => TokenKind::Match,
        "memo" => TokenKind::Memo,
        "module" => TokenKind::Module,
        "mut" => TokenKind::Mut,
        "null" => TokenKind::Null,
        "on_mount" => TokenKind::OnMount,
        "on_update" => TokenKind::OnUpdate,
        "on_unmount" => TokenKind::OnUnmount,
        "parallel" => TokenKind::Parallel,
        "props" => TokenKind::Props,
        "pub" => TokenKind::Pub,
        "return" => TokenKind::Return,
        "script" => TokenKind::Script,
        "self" => TokenKind::SelfKw,
        "signal" => TokenKind::Signal,
        "state" => TokenKind::State,
        "step" => TokenKind::Step,
        "struct" => TokenKind::Struct,
        "style" => TokenKind::Style,
        "super" => TokenKind::Super,
        "trait" => TokenKind::Trait,
        "true" => TokenKind::True,
        "type" => TokenKind::Type,
        "typeof" => TokenKind::Typeof,
        "unsafe" => TokenKind::Unsafe,
        "use" => TokenKind::Use,
        "view" => TokenKind::View,
        "while" => TokenKind::While,
        "yield" => TokenKind::Yield,
        "actor" => TokenKind::Actor,
        "channel" => TokenKind::Channel,
        "receiver" => TokenKind::Receiver,
        "send" => TokenKind::Send,
        "get" => TokenKind::Get,
        "set" => TokenKind::Set,
        _ => return None,
    };
    Some(kind)
}
