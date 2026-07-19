//! Statement parsing.

use crate::Parser;
use five_ast::*;
use five_core::{FiveResult, Span};
use five_lexer::TokenKind;

impl<'src> Parser<'src> {
    /// Parse a statement.
    pub fn parse_stmt(&mut self) -> FiveResult<Stmt> {
        let token = self.peek()?.clone();

        match token.kind {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Fn => self.parse_fn_stmt(),
            TokenKind::Async => {
                // async fn ...
                self.advance()?;
                self.parse_fn_stmt_inner(true, token.span)
            }
            TokenKind::Type => self.parse_type_stmt(),
            TokenKind::Data => self.parse_data_stmt(),
            TokenKind::Struct => self.parse_struct_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Import => self.parse_import_stmt(),
            TokenKind::Export => self.parse_export_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// Parse a let statement: let x = expr or let mut x = expr
    fn parse_let_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Let)?;
        let mutable = self.match_token(TokenKind::Mut)?.is_some();
        let (name, _) = self.parse_identifier()?;

        let type_ann = if self.match_token(TokenKind::Colon)?.is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        let span = start.span.merge(value.span);

        // Optional semicolon
        self.match_token(TokenKind::Semicolon)?;

        Ok(Stmt::new(
            StmtKind::Let {
                name,
                type_ann,
                value,
                mutable,
                ownership: Ownership::Owned,
            },
            span,
        ))
    }

    /// Parse a function statement: fn name(params) -> Type with Effects { body }
    fn parse_fn_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Fn)?;
        self.parse_fn_stmt_inner(false, start.span)
    }

    fn parse_fn_stmt_inner(&mut self, is_async: bool, start_span: Span) -> FiveResult<Stmt> {
        let (name, _) = self.parse_identifier()?;

        // Generic parameters (optional)
        let _generics = if self.match_token(TokenKind::Lt)?.is_some() {
            let mut params = vec![];
            if !self.check(TokenKind::Gt)? {
                let (param_name, _) = self.parse_identifier()?;
                params.push(param_name);
                while self.match_token(TokenKind::Comma)?.is_some() {
                    let (param_name, _) = self.parse_identifier()?;
                    params.push(param_name);
                }
            }
            self.expect(TokenKind::Gt)?;
            params
        } else {
            vec![]
        };

        // Parameters
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        // Return type (optional)
        let return_type = if self.match_token(TokenKind::Arrow)?.is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Effects (optional)
        let effects = if self.match_token(TokenKind::With)?.is_some() {
            self.parse_effects()?
        } else {
            vec![]
        };

        // Body
        self.expect(TokenKind::LBrace)?;
        let body_span = self.peek()?.span;
        let body = self.parse_block_expr(body_span)?;
        let span = start_span.merge(body.span);

        Ok(Stmt::new(
            StmtKind::Fn {
                name,
                params,
                return_type,
                effects,
                body,
                is_async,
            },
            span,
        ))
    }

    /// Parse function parameters.
    fn parse_params(&mut self) -> FiveResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(TokenKind::RParen)? {
            return Ok(params);
        }

        params.push(self.parse_param()?);

        while self.match_token(TokenKind::Comma)?.is_some() {
            if self.check(TokenKind::RParen)? {
                break;
            }
            params.push(self.parse_param()?);
        }

        Ok(params)
    }

    /// Parse a single parameter.
    fn parse_param(&mut self) -> FiveResult<Param> {
        // Check for ownership modifiers
        let (ownership, start_span) = if self.match_token(TokenKind::Ampersand)?.is_some() {
            let span = self.peek()?.span;
            if self.match_token(TokenKind::Mut)?.is_some() {
                (Ownership::MutBorrowed, span)
            } else {
                (Ownership::Borrowed, span)
            }
        } else {
            (Ownership::Owned, self.peek()?.span)
        };

        let (name, name_span) = self.parse_identifier()?;

        let type_ann = if self.match_token(TokenKind::Colon)?.is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        let span = start_span.merge(
            type_ann
                .as_ref()
                .map(|t| t.span)
                .unwrap_or(name_span),
        );

        Ok(Param {
            name,
            type_ann,
            ownership,
            span,
        })
    }

    /// Parse a type alias: type Name<T> = Type
    fn parse_type_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Type)?;
        let (name, _) = self.parse_identifier()?;

        // Generic parameters
        let params = if self.match_token(TokenKind::Lt)?.is_some() {
            let mut params = vec![];
            if !self.check(TokenKind::Gt)? {
                let (param_name, _) = self.parse_identifier()?;
                params.push(param_name);
                while self.match_token(TokenKind::Comma)?.is_some() {
                    let (param_name, _) = self.parse_identifier()?;
                    params.push(param_name);
                }
            }
            self.expect(TokenKind::Gt)?;
            params
        } else {
            vec![]
        };

        self.expect(TokenKind::Eq)?;
        let definition = self.parse_type()?;
        let span = start.span.merge(definition.span);

        self.match_token(TokenKind::Semicolon)?;

        Ok(Stmt::new(
            StmtKind::Type {
                name,
                params,
                definition,
            },
            span,
        ))
    }

    /// Parse an algebraic data type: data Name<T> { Variant1, Variant2(T) }
    fn parse_data_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Data)?;
        let (name, _) = self.parse_identifier()?;

        // Generic parameters
        let params = if self.match_token(TokenKind::Lt)?.is_some() {
            let mut params = vec![];
            if !self.check(TokenKind::Gt)? {
                let (param_name, _) = self.parse_identifier()?;
                params.push(param_name);
                while self.match_token(TokenKind::Comma)?.is_some() {
                    let (param_name, _) = self.parse_identifier()?;
                    params.push(param_name);
                }
            }
            self.expect(TokenKind::Gt)?;
            params
        } else {
            vec![]
        };

        self.expect(TokenKind::LBrace)?;
        let variants = self.parse_variants()?;
        let end = self.expect(TokenKind::RBrace)?;
        let span = start.span.merge(end.span);

        Ok(Stmt::new(
            StmtKind::Data {
                name,
                params,
                variants,
            },
            span,
        ))
    }

    /// Parse data type variants.
    fn parse_variants(&mut self) -> FiveResult<Vec<Variant>> {
        let mut variants = Vec::new();

        while !self.check(TokenKind::RBrace)? && !self.is_at_end()? {
            variants.push(self.parse_variant()?);

            // Optional comma
            self.match_token(TokenKind::Comma)?;
        }

        Ok(variants)
    }

    /// Parse a single variant.
    fn parse_variant(&mut self) -> FiveResult<Variant> {
        let (name, name_span) = self.parse_identifier()?;

        let (fields, span) = if self.match_token(TokenKind::LParen)?.is_some() {
            // Tuple variant
            let mut types = Vec::new();
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
            (VariantFields::Tuple(types), name_span.merge(end.span))
        } else if self.match_token(TokenKind::LBrace)?.is_some() {
            // Named fields variant
            let fields = self.parse_struct_fields()?;
            let end = self.expect(TokenKind::RBrace)?;
            (VariantFields::Named(fields), name_span.merge(end.span))
        } else {
            (VariantFields::Unit, name_span)
        };

        Ok(Variant { name, fields, span })
    }

    /// Parse a struct definition.
    fn parse_struct_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Struct)?;
        let (name, _) = self.parse_identifier()?;

        // Generic parameters
        let params = if self.match_token(TokenKind::Lt)?.is_some() {
            let mut params = vec![];
            if !self.check(TokenKind::Gt)? {
                let (param_name, _) = self.parse_identifier()?;
                params.push(param_name);
                while self.match_token(TokenKind::Comma)?.is_some() {
                    let (param_name, _) = self.parse_identifier()?;
                    params.push(param_name);
                }
            }
            self.expect(TokenKind::Gt)?;
            params
        } else {
            vec![]
        };

        self.expect(TokenKind::LBrace)?;
        let fields = self.parse_struct_fields()?;
        let end = self.expect(TokenKind::RBrace)?;
        let span = start.span.merge(end.span);

        Ok(Stmt::new(
            StmtKind::Struct {
                name,
                params,
                fields,
            },
            span,
        ))
    }

    /// Parse struct fields.
    fn parse_struct_fields(&mut self) -> FiveResult<Vec<StructField>> {
        let mut fields = Vec::new();

        while !self.check(TokenKind::RBrace)? && !self.is_at_end()? {
            let (name, name_span) = self.parse_identifier()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            let span = name_span.merge(ty.span);

            fields.push(StructField { name, ty, span });

            // Optional comma
            self.match_token(TokenKind::Comma)?;
        }

        Ok(fields)
    }

    /// Parse a while loop.
    fn parse_while_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::While)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body_span = self.peek()?.span;
        let body = self.parse_block_expr(body_span)?;
        let span = start.span.merge(body.span);

        Ok(Stmt::new(StmtKind::While { condition, body }, span))
    }

    /// Parse a for loop.
    fn parse_for_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::For)?;
        let (binding, _) = self.parse_identifier()?;
        self.expect(TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let body_span = self.peek()?.span;
        let body = self.parse_block_expr(body_span)?;
        let span = start.span.merge(body.span);

        Ok(Stmt::new(
            StmtKind::For {
                binding,
                iter,
                body,
            },
            span,
        ))
    }

    /// Parse an import statement.
    fn parse_import_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Import)?;

        let mut path = vec![];
        let (first, _) = self.parse_identifier()?;
        path.push(first);

        while self.match_token(TokenKind::ColonColon)?.is_some() {
            let (part, _) = self.parse_identifier()?;
            path.push(part);
        }

        let alias = if self.check(TokenKind::Identifier)? {
            // Check for "as" keyword
            let peek_span = self.peek()?.span;
            let text = self.get_text(peek_span);
            if text == "as" {
                self.advance()?;
                let (alias_name, _) = self.parse_identifier()?;
                Some(alias_name)
            } else {
                None
            }
        } else {
            None
        };

        let span = start.span.merge(self.peek()?.span);
        self.match_token(TokenKind::Semicolon)?;

        Ok(Stmt::new(StmtKind::Import { path, alias }, span))
    }

    /// Parse an export statement.
    fn parse_export_stmt(&mut self) -> FiveResult<Stmt> {
        let start = self.expect(TokenKind::Export)?;
        let inner = self.parse_stmt()?;
        let span = start.span.merge(inner.span);

        Ok(Stmt::new(StmtKind::Export(Box::new(inner)), span))
    }

    /// Parse an expression statement.
    fn parse_expr_stmt(&mut self) -> FiveResult<Stmt> {
        let expr = self.parse_expr()?;
        let span = expr.span;

        // Optional semicolon
        self.match_token(TokenKind::Semicolon)?;

        Ok(Stmt::new(StmtKind::Expr(expr), span))
    }
}
