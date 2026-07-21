//! Expression parsing.

use crate::Parser;
use five_ast::*;
use five_core::{FiveError, FiveResult, Span};
use five_lexer::TokenKind;

impl<'src> Parser<'src> {
    /// Parse an expression.
    pub fn parse_expr(&mut self) -> FiveResult<Expr> {
        self.parse_assignment()
    }

    /// Parse an assignment expression: x = value, x += value, etc.
    fn parse_assignment(&mut self) -> FiveResult<Expr> {
        let expr = self.parse_pipe()?;

        // Check for assignment operators
        if self.check(TokenKind::Eq)? {
            // Make sure the left side is assignable
            if !self.is_valid_assignment_target(&expr) {
                return Err(FiveError::parser(
                    "Invalid assignment target",
                    expr.span,
                ));
            }

            self.advance()?;
            let value = self.parse_assignment()?; // Right-associative
            let span = expr.span.merge(value.span);
            return Ok(Expr::new(
                ExprKind::Assign {
                    target: Box::new(expr),
                    value: Box::new(value),
                },
                span,
            ));
        }

        // Compound assignment operators
        let compound_op = if self.match_token(TokenKind::PlusEq)?.is_some() {
            Some(BinaryOp::Add)
        } else if self.match_token(TokenKind::MinusEq)?.is_some() {
            Some(BinaryOp::Sub)
        } else if self.match_token(TokenKind::StarEq)?.is_some() {
            Some(BinaryOp::Mul)
        } else if self.match_token(TokenKind::SlashEq)?.is_some() {
            Some(BinaryOp::Div)
        } else {
            None
        };

        if let Some(op) = compound_op {
            if !self.is_valid_assignment_target(&expr) {
                return Err(FiveError::parser(
                    "Invalid assignment target",
                    expr.span,
                ));
            }

            let value = self.parse_assignment()?;
            let span = expr.span.merge(value.span);
            return Ok(Expr::new(
                ExprKind::CompoundAssign {
                    target: Box::new(expr),
                    op,
                    value: Box::new(value),
                },
                span,
            ));
        }

        Ok(expr)
    }

    /// Check if an expression is a valid assignment target.
    fn is_valid_assignment_target(&self, expr: &Expr) -> bool {
        matches!(
            expr.kind,
            ExprKind::Identifier(_) | ExprKind::Field { .. } | ExprKind::Index { .. }
        )
    }

    /// Parse a pipe expression: expr |> expr |> expr
    fn parse_pipe(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_or()?;

        while self.match_token(TokenKind::Pipe)?.is_some() {
            let right = self.parse_or()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, BinaryOp::Pipe, right, span);
        }

        Ok(left)
    }

    /// Parse an or expression: a || b
    fn parse_or(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_and()?;

        while self.match_token(TokenKind::OrOr)?.is_some() {
            let right = self.parse_and()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, BinaryOp::Or, right, span);
        }

        Ok(left)
    }

    /// Parse an and expression: a && b
    fn parse_and(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_equality()?;

        while self.match_token(TokenKind::AndAnd)?.is_some() {
            let right = self.parse_equality()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, BinaryOp::And, right, span);
        }

        Ok(left)
    }

    /// Parse an equality expression: a == b, a != b
    fn parse_equality(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = if self.match_token(TokenKind::EqEq)?.is_some() {
                BinaryOp::Eq
            } else if self.match_token(TokenKind::BangEq)?.is_some() {
                BinaryOp::Ne
            } else {
                break;
            };

            let right = self.parse_comparison()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, op, right, span);
        }

        Ok(left)
    }

    /// Parse a comparison expression: a < b, a <= b, etc.
    fn parse_comparison(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_range()?;

        loop {
            let op = if self.match_token(TokenKind::Lt)?.is_some() {
                BinaryOp::Lt
            } else if self.match_token(TokenKind::Gt)?.is_some() {
                BinaryOp::Gt
            } else if self.match_token(TokenKind::LtEq)?.is_some() {
                BinaryOp::Le
            } else if self.match_token(TokenKind::GtEq)?.is_some() {
                BinaryOp::Ge
            } else {
                break;
            };

            let right = self.parse_range()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, op, right, span);
        }

        Ok(left)
    }

    /// Parse a range expression: 1..10, 1..=10
    fn parse_range(&mut self) -> FiveResult<Expr> {
        let start = self.parse_term()?;

        if self.match_token(TokenKind::DotDotEq)?.is_some() {
            let end = self.parse_term()?;
            let span = start.span.merge(end.span);
            return Ok(Expr::new(
                ExprKind::Range {
                    start: Some(Box::new(start)),
                    end: Some(Box::new(end)),
                    inclusive: true,
                },
                span,
            ));
        }

        if self.match_token(TokenKind::DotDot)?.is_some() {
            let end = self.parse_term()?;
            let span = start.span.merge(end.span);
            return Ok(Expr::new(
                ExprKind::Range {
                    start: Some(Box::new(start)),
                    end: Some(Box::new(end)),
                    inclusive: false,
                },
                span,
            ));
        }

        Ok(start)
    }

    /// Parse a term: a + b, a - b
    fn parse_term(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_factor()?;

        loop {
            let op = if self.match_token(TokenKind::Plus)?.is_some() {
                BinaryOp::Add
            } else if self.match_token(TokenKind::Minus)?.is_some() {
                BinaryOp::Sub
            } else {
                break;
            };

            let right = self.parse_factor()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, op, right, span);
        }

        Ok(left)
    }

    /// Parse a factor: a * b, a / b, a % b
    fn parse_factor(&mut self) -> FiveResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = if self.match_token(TokenKind::Star)?.is_some() {
                BinaryOp::Mul
            } else if self.match_token(TokenKind::Slash)?.is_some() {
                BinaryOp::Div
            } else if self.match_token(TokenKind::Percent)?.is_some() {
                BinaryOp::Mod
            } else {
                break;
            };

            let right = self.parse_unary()?;
            let span = left.span.merge(right.span);
            left = Expr::binary(left, op, right, span);
        }

        Ok(left)
    }

    /// Parse a unary expression: -x, !x, &x, &mut x
    fn parse_unary(&mut self) -> FiveResult<Expr> {
        if let Some(token) = self.match_token(TokenKind::Minus)? {
            let expr = self.parse_unary()?;
            let span = token.span.merge(expr.span);
            return Ok(Expr::unary(UnaryOp::Neg, expr, span));
        }

        if let Some(token) = self.match_token(TokenKind::Bang)? {
            let expr = self.parse_unary()?;
            let span = token.span.merge(expr.span);
            return Ok(Expr::unary(UnaryOp::Not, expr, span));
        }

        if let Some(token) = self.match_token(TokenKind::Ampersand)? {
            let is_mut = self.match_token(TokenKind::Mut)?.is_some();
            let expr = self.parse_unary()?;
            let span = token.span.merge(expr.span);
            let op = if is_mut { UnaryOp::MutRef } else { UnaryOp::Ref };
            return Ok(Expr::unary(op, expr, span));
        }

        if let Some(token) = self.match_token(TokenKind::Star)? {
            let expr = self.parse_unary()?;
            let span = token.span.merge(expr.span);
            return Ok(Expr::unary(UnaryOp::Deref, expr, span));
        }

        self.parse_postfix()
    }

    /// Parse postfix expressions: calls, field access, indexing
    fn parse_postfix(&mut self) -> FiveResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenKind::LParen)?.is_some() {
                // Function call
                let args = self.parse_args()?;
                let end = self.expect(TokenKind::RParen)?;
                let span = expr.span.merge(end.span);
                expr = Expr::new(
                    ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    span,
                );
            } else if self.match_token(TokenKind::Dot)?.is_some() {
                // Field access or method call
                let (name, name_span) = self.parse_identifier()?;

                if self.match_token(TokenKind::LParen)?.is_some() {
                    // Method call
                    let args = self.parse_args()?;
                    let end = self.expect(TokenKind::RParen)?;
                    let span = expr.span.merge(end.span);
                    expr = Expr::new(
                        ExprKind::MethodCall {
                            object: Box::new(expr),
                            method: name,
                            args,
                        },
                        span,
                    );
                } else {
                    // Field access
                    let span = expr.span.merge(name_span);
                    expr = Expr::new(
                        ExprKind::Field {
                            object: Box::new(expr),
                            field: name,
                        },
                        span,
                    );
                }
            } else if self.match_token(TokenKind::LBracket)?.is_some() {
                // Index access
                let index = self.parse_expr()?;
                let end = self.expect(TokenKind::RBracket)?;
                let span = expr.span.merge(end.span);
                expr = Expr::new(
                    ExprKind::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse function call arguments.
    fn parse_args(&mut self) -> FiveResult<Vec<Expr>> {
        let mut args = Vec::new();

        if !self.check(TokenKind::RParen)? {
            args.push(self.parse_expr()?);

            while self.match_token(TokenKind::Comma)?.is_some() {
                if self.check(TokenKind::RParen)? {
                    break;
                }
                args.push(self.parse_expr()?);
            }
        }

        Ok(args)
    }

    /// Parse a primary expression.
    fn parse_primary(&mut self) -> FiveResult<Expr> {
        let token = self.peek()?.clone();

        match token.kind {
            // Literals
            TokenKind::Int => {
                self.advance()?;
                let text = self.get_text(token.span).replace('_', "");
                let value: i64 = text.parse().map_err(|_| {
                    FiveError::parser("Invalid integer literal", token.span)
                })?;
                Ok(Expr::literal(Literal::Int(value), token.span))
            }

            TokenKind::Float => {
                self.advance()?;
                let text = self.get_text(token.span).replace('_', "");
                let value: f64 = text.parse().map_err(|_| {
                    FiveError::parser("Invalid float literal", token.span)
                })?;
                Ok(Expr::literal(Literal::Float(value), token.span))
            }

            TokenKind::String => {
                self.advance()?;
                let text = self.get_text(token.span);
                // Check for string interpolation
                self.parse_string_with_interpolation(text, token.span)
            }

            TokenKind::Char => {
                self.advance()?;
                let text = self.get_text(token.span);
                let value = self.parse_char_literal(text)?;
                Ok(Expr::literal(Literal::Char(value), token.span))
            }

            TokenKind::True => {
                self.advance()?;
                Ok(Expr::literal(Literal::Bool(true), token.span))
            }

            TokenKind::False => {
                self.advance()?;
                Ok(Expr::literal(Literal::Bool(false), token.span))
            }

            TokenKind::Nil => {
                self.advance()?;
                Ok(Expr::literal(Literal::Nil, token.span))
            }

            // Identifier
            TokenKind::Identifier => {
                self.advance()?;
                let name = self.get_text(token.span).to_string();

                // Check for struct literal
                if self.check(TokenKind::LBrace)? {
                    // Could be block or struct - peek ahead
                    // For now, only treat as struct if followed by field: value
                    let struct_expr = self.try_parse_struct_literal(&name, token.span)?;
                    if let Some(expr) = struct_expr {
                        return Ok(expr);
                    }
                }

                Ok(Expr::ident(name, token.span))
            }

            // Grouped expression or tuple or lambda
            TokenKind::LParen => {
                self.advance()?;
                self.parse_paren_expr(token.span)
            }

            // Array literal
            TokenKind::LBracket => {
                self.advance()?;
                self.parse_array_literal(token.span)
            }

            // Block expression
            TokenKind::LBrace => {
                self.advance()?;
                self.parse_block_expr(token.span)
            }

            // If expression
            TokenKind::If => {
                self.advance()?;
                self.parse_if_expr(token.span)
            }

            // Match expression
            TokenKind::Match => {
                self.advance()?;
                self.parse_match_expr(token.span)
            }

            // Await expression
            TokenKind::Await => {
                self.advance()?;
                let expr = self.parse_unary()?;
                let span = token.span.merge(expr.span);
                Ok(Expr::new(ExprKind::Await(Box::new(expr)), span))
            }

            // Return expression
            TokenKind::Return => {
                self.advance()?;
                let value = if self.peek_kind()?.starts_expr() {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                let span = if let Some(ref v) = value {
                    token.span.merge(v.span)
                } else {
                    token.span
                };
                Ok(Expr::new(ExprKind::Return(value), span))
            }

            // Break expression
            TokenKind::Break => {
                self.advance()?;
                let value = if self.peek_kind()?.starts_expr() {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                let span = if let Some(ref v) = value {
                    token.span.merge(v.span)
                } else {
                    token.span
                };
                Ok(Expr::new(ExprKind::Break(value), span))
            }

            // Continue expression
            TokenKind::Continue => {
                self.advance()?;
                Ok(Expr::new(ExprKind::Continue, token.span))
            }

            _ => Err(FiveError::parser(
                format!("Unexpected token: {}", token.kind),
                token.span,
            )),
        }
    }

    /// Parse a parenthesized expression, tuple, or lambda.
    fn parse_paren_expr(&mut self, start_span: Span) -> FiveResult<Expr> {
        // Empty parens = unit
        if let Some(end) = self.match_token(TokenKind::RParen)? {
            return Ok(Expr::new(ExprKind::Tuple(vec![]), start_span.merge(end.span)));
        }

        // Check for lambda: (params) => body
        // Try to parse as lambda parameters first
        if let Some(lambda) = self.try_parse_lambda(start_span)? {
            return Ok(lambda);
        }

        // Parse first expression
        let first = self.parse_expr()?;

        // Tuple?
        if self.match_token(TokenKind::Comma)?.is_some() {
            let mut elements = vec![first];
            if !self.check(TokenKind::RParen)? {
                elements.push(self.parse_expr()?);
                while self.match_token(TokenKind::Comma)?.is_some() {
                    if self.check(TokenKind::RParen)? {
                        break;
                    }
                    elements.push(self.parse_expr()?);
                }
            }
            let end = self.expect(TokenKind::RParen)?;
            return Ok(Expr::new(
                ExprKind::Tuple(elements),
                start_span.merge(end.span),
            ));
        }

        // Just a grouped expression
        let _end = self.expect(TokenKind::RParen)?;

        // Check if this is a lambda: (x) => body
        if self.match_token(TokenKind::FatArrow)?.is_some() {
            // This was a single-param lambda
            if let ExprKind::Identifier(name) = &first.kind {
                let param = Param {
                    name: name.clone(),
                    type_ann: None,
                    ownership: Ownership::Owned,
                    span: first.span,
                };
                let body = self.parse_expr()?;
                let span = start_span.merge(body.span);
                return Ok(Expr::new(
                    ExprKind::Lambda {
                        params: vec![param],
                        body: Box::new(body),
                        effects: vec![],
                    },
                    span,
                ));
            }
        }

        Ok(first)
    }

    /// Try to parse a lambda expression.
    fn try_parse_lambda(&mut self, start_span: Span) -> FiveResult<Option<Expr>> {
        // Save position for backtracking
        // This is a simplified check - we look for identifier followed by comma or colon or )=>
        let first_token = self.peek()?.clone();

        if first_token.kind != TokenKind::Identifier {
            return Ok(None);
        }

        // Peek ahead to see if this looks like lambda params
        // We'll try parsing params and if we see =>, it's a lambda
        let mut params = Vec::new();

        // Parse first param
        self.advance()?;
        let first_name = self.get_text(first_token.span).to_string();
        let first_type = if self.match_token(TokenKind::Colon)?.is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        params.push(Param {
            name: first_name,
            type_ann: first_type,
            ownership: Ownership::Owned,
            span: first_token.span,
        });

        // Parse remaining params
        while self.match_token(TokenKind::Comma)?.is_some() {
            if self.check(TokenKind::RParen)? {
                break;
            }
            let (name, name_span) = self.parse_identifier()?;
            let type_ann = if self.match_token(TokenKind::Colon)?.is_some() {
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(Param {
                name,
                type_ann,
                ownership: Ownership::Owned,
                span: name_span,
            });
        }

        // Must have ) =>
        if self.match_token(TokenKind::RParen)?.is_none() {
            // Not a lambda, but we've already consumed tokens...
            // This is a parsing error
            return Err(FiveError::parser(
                "Expected ')' in lambda parameters",
                self.peek()?.span,
            ));
        }

        if self.match_token(TokenKind::FatArrow)?.is_none() {
            // We have (params) but no =>, this should be impossible
            // if we got here since we checked earlier
            return Err(FiveError::parser(
                "Expected '=>' after lambda parameters",
                self.peek()?.span,
            ));
        }

        let body = self.parse_expr()?;
        let span = start_span.merge(body.span);

        Ok(Some(Expr::new(
            ExprKind::Lambda {
                params,
                body: Box::new(body),
                effects: vec![],
            },
            span,
        )))
    }

    /// Parse an array literal.
    fn parse_array_literal(&mut self, start_span: Span) -> FiveResult<Expr> {
        let mut elements = Vec::new();

        if !self.check(TokenKind::RBracket)? {
            elements.push(self.parse_expr()?);

            while self.match_token(TokenKind::Comma)?.is_some() {
                if self.check(TokenKind::RBracket)? {
                    break;
                }
                elements.push(self.parse_expr()?);
            }
        }

        let end = self.expect(TokenKind::RBracket)?;
        Ok(Expr::new(
            ExprKind::Array(elements),
            start_span.merge(end.span),
        ))
    }

    /// Parse a block expression.
    pub fn parse_block_expr(&mut self, start_span: Span) -> FiveResult<Expr> {
        let mut statements = Vec::new();
        let mut final_expr = None;

        while !self.check(TokenKind::RBrace)? && !self.is_at_end()? {
            let stmt = self.parse_stmt()?;

            // Check if this is an expression without semicolon (final expr)
            if self.check(TokenKind::RBrace)? {
                // If it's an expression statement, it becomes the final expr
                if let StmtKind::Expr(expr) = stmt.kind {
                    final_expr = Some(Box::new(expr));
                } else {
                    statements.push(stmt);
                }
            } else {
                statements.push(stmt);
            }
        }

        let end = self.expect(TokenKind::RBrace)?;
        Ok(Expr::new(
            ExprKind::Block {
                statements,
                expr: final_expr,
            },
            start_span.merge(end.span),
        ))
    }

    /// Parse an if expression.
    fn parse_if_expr(&mut self, start_span: Span) -> FiveResult<Expr> {
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let then_span = self.peek()?.span;
        let then_branch = self.parse_block_expr(then_span)?;

        let else_branch = if self.match_token(TokenKind::Else)?.is_some() {
            if self.check(TokenKind::If)? {
                // else if
                self.advance()?;
                let else_span = self.peek()?.span;
                Some(Box::new(self.parse_if_expr(else_span)?))
            } else {
                self.expect(TokenKind::LBrace)?;
                let else_span = self.peek()?.span;
                Some(Box::new(self.parse_block_expr(else_span)?))
            }
        } else {
            None
        };

        let span = if let Some(ref else_br) = else_branch {
            start_span.merge(else_br.span)
        } else {
            start_span.merge(then_branch.span)
        };

        // For if expressions that were parsed with blocks, extract the actual block expression
        // The then_branch is already a block from parse_block_expr
        Ok(Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            },
            span,
        ))
    }

    /// Parse a match expression.
    fn parse_match_expr(&mut self, start_span: Span) -> FiveResult<Expr> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();

        while !self.check(TokenKind::RBrace)? && !self.is_at_end()? {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(TokenKind::If)?.is_some() {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expr()?;
            let span = pattern.span.merge(body.span);

            arms.push(MatchArm {
                pattern,
                guard,
                body: Box::new(body),
                span,
            });

            // Comma is optional but allowed
            self.match_token(TokenKind::Comma)?;
        }

        let end = self.expect(TokenKind::RBrace)?;
        Ok(Expr::new(
            ExprKind::Match {
                expr: Box::new(expr),
                arms,
            },
            start_span.merge(end.span),
        ))
    }

    /// Parse a pattern.
    pub fn parse_pattern(&mut self) -> FiveResult<Pattern> {
        let token = self.peek()?.clone();

        match token.kind {
            // Wildcard
            TokenKind::Identifier if self.get_text(token.span) == "_" => {
                self.advance()?;
                Ok(Pattern {
                    kind: PatternKind::Wildcard,
                    span: token.span,
                })
            }

            // Identifier or Constructor
            TokenKind::Identifier => {
                self.advance()?;
                let name = self.get_text(token.span).to_string();

                // Check for constructor with args
                if self.match_token(TokenKind::LParen)?.is_some() {
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RParen)? {
                        args.push(self.parse_pattern()?);
                        while self.match_token(TokenKind::Comma)?.is_some() {
                            if self.check(TokenKind::RParen)? {
                                break;
                            }
                            args.push(self.parse_pattern()?);
                        }
                    }
                    let end = self.expect(TokenKind::RParen)?;
                    return Ok(Pattern {
                        kind: PatternKind::Constructor { name, args },
                        span: token.span.merge(end.span),
                    });
                }

                // Check if this looks like a constructor (PascalCase)
                if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    Ok(Pattern {
                        kind: PatternKind::Constructor { name, args: vec![] },
                        span: token.span,
                    })
                } else {
                    Ok(Pattern {
                        kind: PatternKind::Identifier(name),
                        span: token.span,
                    })
                }
            }

            // Literal patterns
            TokenKind::Int => {
                self.advance()?;
                let text = self.get_text(token.span).replace('_', "");
                let value: i64 = text.parse().map_err(|_| {
                    FiveError::parser("Invalid integer literal", token.span)
                })?;
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Int(value)),
                    span: token.span,
                })
            }

            TokenKind::String => {
                self.advance()?;
                let text = self.get_text(token.span);
                let value = self.parse_string_literal(text)?;
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::String(value)),
                    span: token.span,
                })
            }

            TokenKind::True => {
                self.advance()?;
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Bool(true)),
                    span: token.span,
                })
            }

            TokenKind::False => {
                self.advance()?;
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Bool(false)),
                    span: token.span,
                })
            }

            TokenKind::Nil => {
                self.advance()?;
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Nil),
                    span: token.span,
                })
            }

            // Tuple pattern
            TokenKind::LParen => {
                self.advance()?;
                let mut patterns = Vec::new();
                if !self.check(TokenKind::RParen)? {
                    patterns.push(self.parse_pattern()?);
                    while self.match_token(TokenKind::Comma)?.is_some() {
                        if self.check(TokenKind::RParen)? {
                            break;
                        }
                        patterns.push(self.parse_pattern()?);
                    }
                }
                let end = self.expect(TokenKind::RParen)?;
                Ok(Pattern {
                    kind: PatternKind::Tuple(patterns),
                    span: token.span.merge(end.span),
                })
            }

            _ => Err(FiveError::parser(
                format!("Unexpected token in pattern: {}", token.kind),
                token.span,
            )),
        }
    }

    /// Try to parse a struct literal.
    fn try_parse_struct_literal(
        &mut self,
        name: &str,
        start_span: Span,
    ) -> FiveResult<Option<Expr>> {
        // Check if this looks like a struct literal: Name { identifier: ...
        // We need to peek 3 tokens: { identifier :
        if !self.check(TokenKind::LBrace)? {
            return Ok(None);
        }

        // Peek at token after opening brace (second token)
        let peek2 = self.peek_second()?;

        if peek2.kind == TokenKind::RBrace {
            // Empty struct: Name { }
            self.expect(TokenKind::LBrace)?;
            let end = self.expect(TokenKind::RBrace)?;
            let span = start_span.merge(end.span);
            return Ok(Some(Expr::new(
                ExprKind::Struct { name: name.to_string(), fields: vec![] },
                span,
            )));
        }

        if peek2.kind != TokenKind::Identifier {
            // Not a struct literal (block or something else)
            return Ok(None);
        }

        // Peek at the third token to see if it's a colon
        let peek3 = self.peek_third()?;
        if peek3.kind != TokenKind::Colon {
            // Not a struct literal - it's something like `x { foo(...) }`
            // which is an identifier followed by a block
            return Ok(None);
        }

        // It IS a struct literal: Name { field: value, ... }
        self.expect(TokenKind::LBrace)?;

        let name = name.to_string();
        let mut fields = Vec::new();

        while !self.check(TokenKind::RBrace)? && !self.is_at_end()? {
            let (field_name, _) = self.parse_identifier()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push((field_name, value));

            // Optional comma
            if self.match_token(TokenKind::Comma)?.is_none() {
                break;
            }
        }

        let end = self.expect(TokenKind::RBrace)?;
        let span = start_span.merge(end.span);

        Ok(Some(Expr::new(
            ExprKind::Struct { name, fields },
            span,
        )))
    }

    /// Peek at the second token ahead (through lexer).
    fn peek_second(&mut self) -> FiveResult<five_lexer::Token> {
        Ok(self.lexer.peek_second()?.clone())
    }

    /// Peek at the third token ahead (through lexer).
    fn peek_third(&mut self) -> FiveResult<five_lexer::Token> {
        Ok(self.lexer.peek_third()?.clone())
    }

    /// Parse a string with potential interpolation: "Hello {name}!"
    fn parse_string_with_interpolation(&mut self, text: &str, span: Span) -> FiveResult<Expr> {
        let inner = &text[1..text.len() - 1]; // Remove quotes

        // Check if there's any interpolation
        if !inner.contains('{') {
            // No interpolation, just parse as regular string
            let value = self.parse_string_literal(text)?;
            return Ok(Expr::literal(Literal::String(value), span));
        }

        // Parse interpolated string into concatenation of parts
        let mut parts: Vec<Expr> = Vec::new();
        let mut current_str = String::new();
        let mut chars = inner.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                // Handle escape sequences
                match chars.next() {
                    Some('n') => current_str.push('\n'),
                    Some('r') => current_str.push('\r'),
                    Some('t') => current_str.push('\t'),
                    Some('\\') => current_str.push('\\'),
                    Some('"') => current_str.push('"'),
                    Some('{') => current_str.push('{'),
                    Some('}') => current_str.push('}'),
                    Some('0') => current_str.push('\0'),
                    Some(other) => {
                        current_str.push('\\');
                        current_str.push(other);
                    }
                    None => current_str.push('\\'),
                }
            } else if c == '{' {
                // Start of interpolation
                // Push current string part if any
                if !current_str.is_empty() {
                    parts.push(Expr::literal(Literal::String(current_str.clone()), span));
                    current_str.clear();
                }

                // Extract expression inside braces
                let mut expr_str = String::new();
                let mut brace_depth = 1;
                while let Some(ec) = chars.next() {
                    if ec == '{' {
                        brace_depth += 1;
                        expr_str.push(ec);
                    } else if ec == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            break;
                        }
                        expr_str.push(ec);
                    } else {
                        expr_str.push(ec);
                    }
                }

                // Parse the expression
                let mut inner_parser = Parser::new(&expr_str);
                let expr = inner_parser.parse_expr()?;

                // Wrap in string() call to convert to string
                let string_call = Expr::new(
                    ExprKind::Call {
                        callee: Box::new(Expr::ident("string", span)),
                        args: vec![expr],
                    },
                    span,
                );
                parts.push(string_call);
            } else {
                current_str.push(c);
            }
        }

        // Push remaining string part
        if !current_str.is_empty() {
            parts.push(Expr::literal(Literal::String(current_str), span));
        }

        // If no parts, return empty string
        if parts.is_empty() {
            return Ok(Expr::literal(Literal::String(String::new()), span));
        }

        // If one part, return it directly
        if parts.len() == 1 {
            return Ok(parts.remove(0));
        }

        // Combine parts with + operator
        let mut result = parts.remove(0);
        for part in parts {
            result = Expr::new(
                ExprKind::Binary {
                    left: Box::new(result),
                    op: BinaryOp::Add,
                    right: Box::new(part),
                },
                span,
            );
        }

        Ok(result)
    }

    /// Parse a string literal, handling escape sequences.
    fn parse_string_literal(&self, text: &str) -> FiveResult<String> {
        // Remove surrounding quotes
        let inner = &text[1..text.len() - 1];
        let mut result = String::new();
        let mut chars = inner.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('0') => result.push('\0'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }

    /// Parse a character literal.
    fn parse_char_literal(&self, text: &str) -> FiveResult<char> {
        let inner = &text[1..text.len() - 1];

        if inner.starts_with('\\') {
            match inner.chars().nth(1) {
                Some('n') => Ok('\n'),
                Some('r') => Ok('\r'),
                Some('t') => Ok('\t'),
                Some('\\') => Ok('\\'),
                Some('\'') => Ok('\''),
                Some('0') => Ok('\0'),
                _ => inner.chars().next().ok_or_else(|| {
                    FiveError::parser("Empty character literal", Span::dummy())
                }),
            }
        } else {
            inner.chars().next().ok_or_else(|| {
                FiveError::parser("Empty character literal", Span::dummy())
            })
        }
    }

    /// Parse a type annotation.
    pub fn parse_type(&mut self) -> FiveResult<Type> {
        let token = self.peek()?.clone();

        match token.kind {
            TokenKind::Any => {
                self.advance()?;
                Ok(Type::any(token.span))
            }

            TokenKind::LParen => {
                self.advance()?;
                // Tuple type or function type or unit
                if let Some(end) = self.match_token(TokenKind::RParen)? {
                    // Check for function type: () -> T
                    if self.match_token(TokenKind::Arrow)?.is_some() {
                        let return_type = self.parse_type()?;
                        let effects = if self.match_token(TokenKind::With)?.is_some() {
                            self.parse_effects()?
                        } else {
                            vec![]
                        };
                        let span = token.span.merge(return_type.span);
                        return Ok(Type::new(
                            TypeKind::Function {
                                params: vec![],
                                return_type: Box::new(return_type),
                                effects,
                            },
                            span,
                        ));
                    }
                    return Ok(Type::unit(token.span.merge(end.span)));
                }

                let first = self.parse_type()?;

                if self.match_token(TokenKind::Comma)?.is_some() {
                    // Tuple type
                    let mut types = vec![first];
                    if !self.check(TokenKind::RParen)? {
                        types.push(self.parse_type()?);
                        while self.match_token(TokenKind::Comma)?.is_some() {
                            if self.check(TokenKind::RParen)? {
                                break;
                            }
                            types.push(self.parse_type()?);
                        }
                    }
                    let end = self.expect(TokenKind::RParen)?;

                    // Check for function type
                    if self.match_token(TokenKind::Arrow)?.is_some() {
                        let return_type = self.parse_type()?;
                        let effects = if self.match_token(TokenKind::With)?.is_some() {
                            self.parse_effects()?
                        } else {
                            vec![]
                        };
                        let span = token.span.merge(return_type.span);
                        return Ok(Type::new(
                            TypeKind::Function {
                                params: types,
                                return_type: Box::new(return_type),
                                effects,
                            },
                            span,
                        ));
                    }

                    Ok(Type::new(
                        TypeKind::Tuple(types),
                        token.span.merge(end.span),
                    ))
                } else {
                    let _end = self.expect(TokenKind::RParen)?;

                    // Check for function type: (T) -> U
                    if self.match_token(TokenKind::Arrow)?.is_some() {
                        let return_type = self.parse_type()?;
                        let effects = if self.match_token(TokenKind::With)?.is_some() {
                            self.parse_effects()?
                        } else {
                            vec![]
                        };
                        let span = token.span.merge(return_type.span);
                        return Ok(Type::new(
                            TypeKind::Function {
                                params: vec![first],
                                return_type: Box::new(return_type),
                                effects,
                            },
                            span,
                        ));
                    }

                    // Single-element tuple is just the type
                    Ok(first)
                }
            }

            TokenKind::LBracket => {
                self.advance()?;
                let inner = self.parse_type()?;
                let end = self.expect(TokenKind::RBracket)?;
                Ok(Type::new(
                    TypeKind::Array(Box::new(inner)),
                    token.span.merge(end.span),
                ))
            }

            TokenKind::Ampersand => {
                self.advance()?;
                let mutable = self.match_token(TokenKind::Mut)?.is_some();
                let inner = self.parse_type()?;
                let span = token.span.merge(inner.span);
                Ok(Type::new(
                    TypeKind::Reference {
                        inner: Box::new(inner),
                        mutable,
                        lifetime: None,
                    },
                    span,
                ))
            }

            TokenKind::Identifier => {
                self.advance()?;
                let name = self.get_text(token.span).to_string();

                // Check for generic params
                if self.match_token(TokenKind::Lt)?.is_some() {
                    let mut params = vec![self.parse_type()?];
                    while self.match_token(TokenKind::Comma)?.is_some() {
                        params.push(self.parse_type()?);
                    }
                    let end = self.expect(TokenKind::Gt)?;
                    Ok(Type::new(
                        TypeKind::Generic { name, params },
                        token.span.merge(end.span),
                    ))
                } else {
                    Ok(Type::named(name, token.span))
                }
            }

            _ => Err(FiveError::parser(
                format!("Expected type, found {}", token.kind),
                token.span,
            )),
        }
    }

    /// Parse effect annotations.
    pub fn parse_effects(&mut self) -> FiveResult<Vec<Effect>> {
        let mut effects = vec![self.parse_effect()?];

        while self.match_token(TokenKind::Comma)?.is_some() {
            effects.push(self.parse_effect()?);
        }

        Ok(effects)
    }

    /// Parse a single effect.
    fn parse_effect(&mut self) -> FiveResult<Effect> {
        let token = self.expect(TokenKind::Identifier)?;
        let name = self.get_text(token.span);

        Ok(match name {
            "IO" => Effect::IO,
            "State" => Effect::State,
            "Async" => Effect::Async,
            "Pure" => Effect::Pure,
            _ => Effect::Custom(name.to_string()),
        })
    }
}
