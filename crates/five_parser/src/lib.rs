//! Five Parser - Recursive descent parser with Pratt parsing for expressions.

mod expr;
mod stmt;

use five_ast::Program;
use five_core::{FiveError, FiveResult, Span};
use five_lexer::{Lexer, Token, TokenKind};

/// The Five parser.
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    source: &'src str,
}

impl<'src> Parser<'src> {
    /// Create a new parser for the given source code.
    pub fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
            source,
        }
    }

    /// Parse the source code into a program.
    pub fn parse(source: &str) -> FiveResult<Program> {
        let mut parser = Parser::new(source);
        parser.parse_program()
    }

    /// Parse a complete program.
    pub fn parse_program(&mut self) -> FiveResult<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end()? {
            statements.push(self.parse_stmt()?);
        }

        Ok(Program::new(statements))
    }

    // Helper methods

    /// Check if we're at the end of input.
    fn is_at_end(&mut self) -> FiveResult<bool> {
        self.lexer.is_at_end()
    }

    /// Peek at the current token.
    fn peek(&mut self) -> FiveResult<&Token> {
        self.lexer.peek()
    }

    /// Get the current token kind.
    fn peek_kind(&mut self) -> FiveResult<TokenKind> {
        Ok(self.peek()?.kind)
    }

    /// Advance to the next token.
    fn advance(&mut self) -> FiveResult<Token> {
        self.lexer.next_token()
    }

    /// Check if the current token matches the given kind.
    fn check(&mut self, kind: TokenKind) -> FiveResult<bool> {
        Ok(self.peek()?.kind == kind)
    }

    /// Consume the current token if it matches, otherwise error.
    fn expect(&mut self, kind: TokenKind) -> FiveResult<Token> {
        let token = self.peek()?;
        if token.kind == kind {
            self.advance()
        } else {
            Err(FiveError::parser(
                format!("Expected {}, found {}", kind, token.kind),
                token.span,
            ))
        }
    }

    /// Consume the current token if it matches.
    fn match_token(&mut self, kind: TokenKind) -> FiveResult<Option<Token>> {
        if self.check(kind)? {
            Ok(Some(self.advance()?))
        } else {
            Ok(None)
        }
    }

    /// Get the text for a span.
    fn get_text(&self, span: Span) -> &'src str {
        self.lexer.get_text(span)
    }

    /// Parse an identifier.
    fn parse_identifier(&mut self) -> FiveResult<(String, Span)> {
        let token = self.expect(TokenKind::Identifier)?;
        let name = self.get_text(token.span).to_string();
        Ok((name, token.span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_ast::{ExprKind, Literal, StmtKind};

    #[test]
    fn test_parse_let() {
        let program = Parser::parse("let x = 42").unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StmtKind::Let { name, value, .. } => {
                assert_eq!(name, "x");
                match &value.kind {
                    ExprKind::Literal(Literal::Int(n)) => assert_eq!(*n, 42),
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_function() {
        let program = Parser::parse("fn add(a, b) { a + b }").unwrap();
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0].kind {
            StmtKind::Fn { name, params, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_binary_expr() {
        let program = Parser::parse("1 + 2 * 3").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_pipe() {
        let program = Parser::parse("input |> process |> output").unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
