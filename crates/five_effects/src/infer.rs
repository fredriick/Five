//! Effect inference.

use crate::EffectSet;
use five_ast::*;
use five_core::{FiveError, FiveResult};
use std::collections::HashMap;

/// Effect checker and inference engine.
pub struct EffectChecker {
    /// Known function effects
    function_effects: HashMap<String, EffectSet>,
}

impl Default for EffectChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectChecker {
    pub fn new() -> Self {
        let mut function_effects = HashMap::new();

        // Built-in functions with effects
        let mut io_effects = EffectSet::new();
        io_effects.add(Effect::IO);

        function_effects.insert("print".to_string(), io_effects.clone());
        function_effects.insert("println".to_string(), io_effects.clone());
        function_effects.insert("input".to_string(), io_effects);

        Self { function_effects }
    }

    /// Check a program for effect consistency.
    pub fn check_program(&mut self, program: &Program) -> FiveResult<()> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    /// Check a statement.
    fn check_stmt(&mut self, stmt: &Stmt) -> FiveResult<EffectSet> {
        match &stmt.kind {
            StmtKind::Let { value, .. } => self.infer_expr(value),

            StmtKind::Fn {
                name,
                body,
                effects,
                ..
            } => {
                // Infer effects from body
                let inferred = self.infer_expr(body)?;

                // Check declared effects are sufficient
                let declared: EffectSet = effects.clone().into();

                if !inferred.is_subset_of(&declared) {
                    return Err(FiveError::effect(
                        format!(
                            "Function '{}' has undeclared effects: inferred {}, declared {}",
                            name, inferred, declared
                        ),
                        stmt.span,
                    ));
                }

                // Register function effects
                self.function_effects.insert(name.clone(), declared);

                Ok(EffectSet::pure())
            }

            StmtKind::Expr(expr) => self.infer_expr(expr),

            StmtKind::While { condition, body } => {
                let mut effects = self.infer_expr(condition)?;
                effects.merge(&self.infer_expr(body)?);
                Ok(effects)
            }

            StmtKind::For { iter, body, .. } => {
                let mut effects = self.infer_expr(iter)?;
                effects.merge(&self.infer_expr(body)?);
                Ok(effects)
            }

            StmtKind::Data { .. } | StmtKind::Struct { .. } | StmtKind::Type { .. } => {
                Ok(EffectSet::pure())
            }

            StmtKind::Import { .. } => Ok(EffectSet::pure()),

            StmtKind::Export(inner) => self.check_stmt(inner),
        }
    }

    /// Infer effects from an expression.
    pub fn infer_expr(&self, expr: &Expr) -> FiveResult<EffectSet> {
        match &expr.kind {
            ExprKind::Literal(_) => Ok(EffectSet::pure()),

            ExprKind::Identifier(_) => Ok(EffectSet::pure()),

            ExprKind::Binary { left, right, .. } => {
                let mut effects = self.infer_expr(left)?;
                effects.merge(&self.infer_expr(right)?);
                Ok(effects)
            }

            ExprKind::Unary { expr: inner, .. } => self.infer_expr(inner),

            ExprKind::Call { callee, args } => {
                let mut effects = EffectSet::new();

                // Get function effects
                if let ExprKind::Identifier(name) = &callee.kind {
                    if let Some(fn_effects) = self.function_effects.get(name) {
                        effects.merge(fn_effects);
                    }
                }

                // Add effects from evaluating arguments
                for arg in args {
                    effects.merge(&self.infer_expr(arg)?);
                }

                Ok(effects)
            }

            ExprKind::MethodCall { object, args, .. } => {
                let mut effects = self.infer_expr(object)?;
                for arg in args {
                    effects.merge(&self.infer_expr(arg)?);
                }
                Ok(effects)
            }

            ExprKind::Field { object, .. } => self.infer_expr(object),

            ExprKind::Index { object, index } => {
                let mut effects = self.infer_expr(object)?;
                effects.merge(&self.infer_expr(index)?);
                Ok(effects)
            }

            ExprKind::Lambda { body, effects, .. } => {
                // Lambda body effects should be subset of declared
                let inferred = self.infer_expr(body)?;
                let declared: EffectSet = effects.clone().into();

                if !inferred.is_subset_of(&declared) {
                    return Err(FiveError::effect(
                        format!(
                            "Lambda has undeclared effects: inferred {}, declared {}",
                            inferred, declared
                        ),
                        expr.span,
                    ));
                }

                Ok(EffectSet::pure()) // Creating a lambda is pure
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut effects = self.infer_expr(condition)?;
                effects.merge(&self.infer_expr(then_branch)?);
                if let Some(else_br) = else_branch {
                    effects.merge(&self.infer_expr(else_br)?);
                }
                Ok(effects)
            }

            ExprKind::Match { expr: scrutinee, arms } => {
                let mut effects = self.infer_expr(scrutinee)?;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        effects.merge(&self.infer_expr(guard)?);
                    }
                    effects.merge(&self.infer_expr(&arm.body)?);
                }
                Ok(effects)
            }

            ExprKind::Block { statements, expr } => {
                let mut effects = EffectSet::new();
                for stmt in statements {
                    // Note: We can't call check_stmt here because it might modify state
                    // For now, just infer from expressions
                    if let StmtKind::Expr(e) = &stmt.kind {
                        effects.merge(&self.infer_expr(e)?);
                    }
                }
                if let Some(e) = expr {
                    effects.merge(&self.infer_expr(e)?);
                }
                Ok(effects)
            }

            ExprKind::Array(elements) => {
                let mut effects = EffectSet::new();
                for elem in elements {
                    effects.merge(&self.infer_expr(elem)?);
                }
                Ok(effects)
            }

            ExprKind::Tuple(elements) => {
                let mut effects = EffectSet::new();
                for elem in elements {
                    effects.merge(&self.infer_expr(elem)?);
                }
                Ok(effects)
            }

            ExprKind::Await(inner) => {
                let mut effects = self.infer_expr(inner)?;
                effects.add(Effect::Async);
                Ok(effects)
            }

            ExprKind::Return(inner) => {
                if let Some(e) = inner {
                    self.infer_expr(e)
                } else {
                    Ok(EffectSet::pure())
                }
            }

            ExprKind::Break(inner) => {
                if let Some(e) = inner {
                    self.infer_expr(e)
                } else {
                    Ok(EffectSet::pure())
                }
            }

            ExprKind::Continue => Ok(EffectSet::pure()),

            ExprKind::Range { start, end, .. } => {
                let mut effects = EffectSet::new();
                if let Some(s) = start {
                    effects.merge(&self.infer_expr(s)?);
                }
                if let Some(e) = end {
                    effects.merge(&self.infer_expr(e)?);
                }
                Ok(effects)
            }

            ExprKind::Struct { fields, .. } => {
                let mut effects = EffectSet::new();
                for (_, value) in fields {
                    effects.merge(&self.infer_expr(value)?);
                }
                Ok(effects)
            }

            ExprKind::Assign { target, value } => {
                let mut effects = self.infer_expr(target)?;
                effects.merge(&self.infer_expr(value)?);
                effects.add(Effect::State); // Assignment has State effect
                Ok(effects)
            }

            ExprKind::CompoundAssign { target, value, .. } => {
                let mut effects = self.infer_expr(target)?;
                effects.merge(&self.infer_expr(value)?);
                effects.add(Effect::State);
                Ok(effects)
            }
        }
    }
}
