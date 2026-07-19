//! Token definitions for the Five lexer.

use five_core::Span;
use logos::Logos;

/// A token in the Five language.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The kind of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Logos)]
#[logos(skip r"[ \t\n\r]+")]
pub enum TokenKind {
    // Keywords
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("fn")]
    Fn,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("type")]
    Type,
    #[token("data")]
    Data,
    #[token("struct")]
    Struct,
    #[token("with")]
    With,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("any")]
    Any,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,
    #[token("|>")]
    Pipe,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("=")]
    Eq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("&")]
    Ampersand,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("|")]
    Bar,

    // Literals
    #[regex(r"[0-9][0-9_]*")]
    Int,
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*")]
    Float,
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,
    #[regex(r"'([^'\\]|\\.)'")]
    Char,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Lifetime
    #[regex(r"'[a-zA-Z_][a-zA-Z0-9_]*")]
    Lifetime,

    // Comments (will be skipped by lexer)
    #[regex(r"//[^\n]*")]
    LineComment,
    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    BlockComment,

    // End of file
    Eof,
}

impl TokenKind {
    /// Check if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Let
                | Self::Mut
                | Self::Fn
                | Self::If
                | Self::Else
                | Self::Match
                | Self::Type
                | Self::Data
                | Self::Struct
                | Self::With
                | Self::Async
                | Self::Await
                | Self::Import
                | Self::Export
                | Self::True
                | Self::False
                | Self::Nil
                | Self::Return
                | Self::Break
                | Self::Continue
                | Self::While
                | Self::For
                | Self::In
                | Self::Any
        )
    }

    /// Check if this token starts an expression.
    pub fn starts_expr(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Float
                | Self::String
                | Self::Char
                | Self::True
                | Self::False
                | Self::Nil
                | Self::Identifier
                | Self::LParen
                | Self::LBracket
                | Self::LBrace
                | Self::If
                | Self::Match
                | Self::Bang
                | Self::Minus
                | Self::Ampersand
                | Self::Await
                | Self::Return
                | Self::Break
                | Self::Continue
        )
    }

    /// Get the string representation of this token kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Let => "let",
            Self::Mut => "mut",
            Self::Fn => "fn",
            Self::If => "if",
            Self::Else => "else",
            Self::Match => "match",
            Self::Type => "type",
            Self::Data => "data",
            Self::Struct => "struct",
            Self::With => "with",
            Self::Async => "async",
            Self::Await => "await",
            Self::Import => "import",
            Self::Export => "export",
            Self::True => "true",
            Self::False => "false",
            Self::Nil => "nil",
            Self::Return => "return",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::While => "while",
            Self::For => "for",
            Self::In => "in",
            Self::Any => "any",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::EqEq => "==",
            Self::BangEq => "!=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
            Self::AndAnd => "&&",
            Self::OrOr => "||",
            Self::Bang => "!",
            Self::Pipe => "|>",
            Self::Arrow => "->",
            Self::FatArrow => "=>",
            Self::Eq => "=",
            Self::PlusEq => "+=",
            Self::MinusEq => "-=",
            Self::StarEq => "*=",
            Self::SlashEq => "/=",
            Self::Ampersand => "&",
            Self::DotDot => "..",
            Self::DotDotEq => "..=",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::ColonColon => "::",
            Self::Semicolon => ";",
            Self::Dot => ".",
            Self::Bar => "|",
            Self::Int => "integer",
            Self::Float => "float",
            Self::String => "string",
            Self::Char => "char",
            Self::Identifier => "identifier",
            Self::Lifetime => "lifetime",
            Self::LineComment => "// comment",
            Self::BlockComment => "/* comment */",
            Self::Eof => "end of file",
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
