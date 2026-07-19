//! Five Lexer - Tokenization for the Five programming language.

mod token;

pub use token::{Token, TokenKind};

use five_core::{FiveError, FiveResult, Span};
use logos::Logos;

/// The Five lexer.
pub struct Lexer<'src> {
    source: &'src str,
    inner: logos::Lexer<'src, TokenKind>,
    peeked: Option<Token>,
    peeked2: Option<Token>,
    peeked3: Option<Token>,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source code.
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            inner: TokenKind::lexer(source),
            peeked: None,
            peeked2: None,
            peeked3: None,
        }
    }

    /// Get the source code.
    pub fn source(&self) -> &'src str {
        self.source
    }

    /// Peek at the next token without consuming it.
    pub fn peek(&mut self) -> FiveResult<&Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.read_next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    /// Peek at the second token ahead without consuming.
    pub fn peek_second(&mut self) -> FiveResult<&Token> {
        // Ensure first token is peeked
        if self.peeked.is_none() {
            self.peeked = Some(self.read_next_token()?);
        }
        // Ensure second token is peeked
        if self.peeked2.is_none() {
            self.peeked2 = Some(self.read_next_token()?);
        }
        Ok(self.peeked2.as_ref().unwrap())
    }

    /// Peek at the third token ahead without consuming.
    pub fn peek_third(&mut self) -> FiveResult<&Token> {
        // Ensure first two tokens are peeked
        self.peek_second()?;
        // Ensure third token is peeked
        if self.peeked3.is_none() {
            self.peeked3 = Some(self.read_next_token()?);
        }
        Ok(self.peeked3.as_ref().unwrap())
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> FiveResult<Token> {
        if let Some(token) = self.peeked.take() {
            // Shift peeked2 to peeked, peeked3 to peeked2
            self.peeked = self.peeked2.take();
            self.peeked2 = self.peeked3.take();
            return Ok(token);
        }
        self.read_next_token()
    }

    /// Read the next token from the underlying lexer.
    fn read_next_token(&mut self) -> FiveResult<Token> {

        loop {
            match self.inner.next() {
                Some(Ok(kind)) => {
                    let span = self.inner.span();
                    let token = Token {
                        kind,
                        span: Span::new(span.start, span.end),
                    };

                    // Skip comments
                    if matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment) {
                        continue;
                    }

                    return Ok(token);
                }
                Some(Err(())) => {
                    let span = self.inner.span();
                    return Err(FiveError::lexer(
                        format!("Unexpected character: {:?}", &self.source[span.clone()]),
                        Span::new(span.start, span.end),
                    ));
                }
                None => {
                    let end = self.source.len();
                    return Ok(Token {
                        kind: TokenKind::Eof,
                        span: Span::new(end, end),
                    });
                }
            }
        }
    }

    /// Check if we're at the end of the input.
    pub fn is_at_end(&mut self) -> FiveResult<bool> {
        Ok(self.peek()?.kind == TokenKind::Eof)
    }

    /// Get the text for a span.
    pub fn get_text(&self, span: Span) -> &'src str {
        &self.source[span.start..span.end]
    }

    /// Tokenize the entire source into a vector of tokens.
    pub fn tokenize(source: &str) -> FiveResult<Vec<Token>> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let source = "let x = 42";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::Int);
        assert_eq!(tokens[4].kind, TokenKind::Eof);
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / |> -> =>";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Pipe);
        assert_eq!(tokens[5].kind, TokenKind::Arrow);
        assert_eq!(tokens[6].kind, TokenKind::FatArrow);
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello world""#;
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::String);
    }

    #[test]
    fn test_comments_are_skipped() {
        let source = "let // comment\nx";
        let tokens = Lexer::tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }
}
