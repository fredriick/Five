//! Type inference engine.

use crate::{InferType, TypeEnv, TypeVar, Unifier};
use five_ast::*;
use five_core::{FiveError, FiveResult, Span};
use std::collections::HashMap;

/// The type checker and inference engine.
pub struct TypeChecker {
    /// Counter for generating fresh type variables
    var_counter: usize,
    /// Type substitutions
    substitutions: HashMap<TypeVar, InferType>,
    /// Current type environment
    env: TypeEnv,
    /// Collected errors
    errors: Vec<FiveError>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut env = TypeEnv::new();

        // Register built-in types
        env.define("Int".to_string(), InferType::Concrete("Int".to_string()));
        env.define("Float".to_string(), InferType::Concrete("Float".to_string()));
        env.define("String".to_string(), InferType::Concrete("String".to_string()));
        env.define("Bool".to_string(), InferType::Concrete("Bool".to_string()));
        env.define("Char".to_string(), InferType::Concrete("Char".to_string()));

        // Register built-in functions
        env.define(
            "print".to_string(),
            InferType::Function {
                params: vec![InferType::Any],
                return_type: Box::new(InferType::Unit),
                effects: vec![Effect::IO],
            },
        );
        env.define(
            "println".to_string(),
            InferType::Function {
                params: vec![InferType::Any],
                return_type: Box::new(InferType::Unit),
                effects: vec![Effect::IO],
            },
        );

        Self {
            var_counter: 0,
            substitutions: HashMap::new(),
            env,
            errors: Vec::new(),
        }
    }

    /// Generate a fresh type variable.
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.var_counter);
        self.var_counter += 1;
        var
    }

    /// Generate a fresh type variable as an InferType.
    pub fn fresh_type(&mut self) -> InferType {
        InferType::Var(self.fresh_var())
    }

    /// Check a program.
    pub fn check_program(&mut self, program: &Program) -> FiveResult<()> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            // Return first error for now
            Err(self.errors.remove(0))
        }
    }

    /// Check a statement.
    pub fn check_stmt(&mut self, stmt: &Stmt) -> FiveResult<InferType> {
        match &stmt.kind {
            StmtKind::Let {
                name,
                type_ann,
                value,
                ..
            } => {
                let inferred = self.infer_expr(value)?;

                if let Some(ann) = type_ann {
                    let expected = InferType::from_ast(ann);
                    self.unify(&expected, &inferred, stmt.span)?;
                }

                let resolved = self.resolve(&inferred);
                self.env.define(name.clone(), resolved);

                Ok(InferType::Unit)
            }

            StmtKind::Fn {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                // Create parameter types
                let param_types: Vec<InferType> = params
                    .iter()
                    .map(|p| {
                        p.type_ann
                            .as_ref()
                            .map(InferType::from_ast)
                            .unwrap_or_else(|| self.fresh_type())
                    })
                    .collect();

                let ret_type = return_type
                    .as_ref()
                    .map(InferType::from_ast)
                    .unwrap_or_else(|| self.fresh_type());

                // Define function type before checking body (for recursion)
                let fn_type = InferType::Function {
                    params: param_types.clone(),
                    return_type: Box::new(ret_type.clone()),
                    effects: vec![], // TODO: infer effects
                };
                self.env.define(name.clone(), fn_type.clone());

                // Check body with parameters in scope
                let old_env = std::mem::take(&mut self.env);
                self.env = TypeEnv::with_parent(old_env);

                for (param, param_type) in params.iter().zip(&param_types) {
                    self.env.define(param.name.clone(), param_type.clone());
                }

                let body_type = self.infer_expr(body)?;
                self.unify(&ret_type, &body_type, body.span)?;

                // Restore environment
                if let Some(parent) = std::mem::take(&mut self.env).parent {
                    self.env = *parent;
                }

                Ok(fn_type)
            }

            StmtKind::Data { name, variants, params, .. } => {
                // Register the data type
                let data_type = if params.is_empty() {
                    InferType::Concrete(name.clone())
                } else {
                    InferType::Generic {
                        name: name.clone(),
                        params: params
                            .iter()
                            .map(|p| InferType::Concrete(p.clone()))
                            .collect(),
                    }
                };

                // Register constructors
                for variant in variants {
                    let constructor_type = match &variant.fields {
                        VariantFields::Unit => data_type.clone(),
                        VariantFields::Tuple(types) => InferType::Function {
                            params: types.iter().map(InferType::from_ast).collect(),
                            return_type: Box::new(data_type.clone()),
                            effects: vec![],
                        },
                        VariantFields::Named(fields) => InferType::Function {
                            params: fields.iter().map(|f| InferType::from_ast(&f.ty)).collect(),
                            return_type: Box::new(data_type.clone()),
                            effects: vec![],
                        },
                    };
                    self.env.define(variant.name.clone(), constructor_type);
                }

                Ok(InferType::Unit)
            }

            StmtKind::Struct { name, fields, params, .. } => {
                let struct_type = if params.is_empty() {
                    InferType::Concrete(name.clone())
                } else {
                    InferType::Generic {
                        name: name.clone(),
                        params: params
                            .iter()
                            .map(|p| InferType::Concrete(p.clone()))
                            .collect(),
                    }
                };

                // Register struct constructor
                let constructor_type = InferType::Function {
                    params: fields.iter().map(|f| InferType::from_ast(&f.ty)).collect(),
                    return_type: Box::new(struct_type),
                    effects: vec![],
                };
                self.env.define(name.clone(), constructor_type);

                Ok(InferType::Unit)
            }

            StmtKind::Type { name, definition, .. } => {
                let ty = InferType::from_ast(definition);
                self.env.define(name.clone(), ty);
                Ok(InferType::Unit)
            }

            StmtKind::Expr(expr) => self.infer_expr(expr),

            StmtKind::While { condition, body } => {
                let cond_type = self.infer_expr(condition)?;
                self.unify(
                    &InferType::Concrete("Bool".to_string()),
                    &cond_type,
                    condition.span,
                )?;
                self.infer_expr(body)?;
                Ok(InferType::Unit)
            }

            StmtKind::For { binding, iter, body } => {
                let _iter_type = self.infer_expr(iter)?;
                // Infer element type from iterator
                let elem_type = self.fresh_type();

                let old_env = std::mem::take(&mut self.env);
                self.env = TypeEnv::with_parent(old_env);
                self.env.define(binding.clone(), elem_type);

                self.infer_expr(body)?;

                if let Some(parent) = std::mem::take(&mut self.env).parent {
                    self.env = *parent;
                }

                Ok(InferType::Unit)
            }

            StmtKind::Import { .. } => Ok(InferType::Unit),
            StmtKind::Export(inner) => self.check_stmt(inner),
        }
    }

    /// Infer the type of an expression.
    pub fn infer_expr(&mut self, expr: &Expr) -> FiveResult<InferType> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Identifier(name) => {
                self.env.get(name).ok_or_else(|| {
                    FiveError::type_error(format!("Undefined variable: {}", name), expr.span)
                })
            }

            ExprKind::Binary { left, op, right } => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        self.unify(&left_type, &right_type, expr.span)?;
                        Ok(left_type)
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        self.unify(&left_type, &right_type, expr.span)?;
                        Ok(InferType::Concrete("Bool".to_string()))
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        self.unify(&InferType::Concrete("Bool".to_string()), &left_type, left.span)?;
                        self.unify(&InferType::Concrete("Bool".to_string()), &right_type, right.span)?;
                        Ok(InferType::Concrete("Bool".to_string()))
                    }
                    BinaryOp::Pipe => {
                        // left |> right means right(left)
                        let result_type = self.fresh_type();
                        let expected_fn = InferType::Function {
                            params: vec![left_type],
                            return_type: Box::new(result_type.clone()),
                            effects: vec![],
                        };
                        self.unify(&expected_fn, &right_type, right.span)?;
                        Ok(result_type)
                    }
                }
            }

            ExprKind::Unary { op, expr: inner } => {
                let inner_type = self.infer_expr(inner)?;

                match op {
                    UnaryOp::Neg => {
                        // Numeric types only
                        Ok(inner_type)
                    }
                    UnaryOp::Not => {
                        self.unify(
                            &InferType::Concrete("Bool".to_string()),
                            &inner_type,
                            inner.span,
                        )?;
                        Ok(InferType::Concrete("Bool".to_string()))
                    }
                    UnaryOp::Ref => Ok(InferType::Reference {
                        inner: Box::new(inner_type),
                        mutable: false,
                    }),
                    UnaryOp::MutRef => Ok(InferType::Reference {
                        inner: Box::new(inner_type),
                        mutable: true,
                    }),
                    UnaryOp::Deref => {
                        let inner_elem = self.fresh_type();
                        self.unify(
                            &InferType::Reference {
                                inner: Box::new(inner_elem.clone()),
                                mutable: false,
                            },
                            &inner_type,
                            inner.span,
                        )?;
                        Ok(inner_elem)
                    }
                }
            }

            ExprKind::Call { callee, args } => {
                let callee_type = self.infer_expr(callee)?;
                let arg_types: Vec<InferType> = args
                    .iter()
                    .map(|a| self.infer_expr(a))
                    .collect::<FiveResult<_>>()?;

                let result_type = self.fresh_type();
                let expected_fn = InferType::Function {
                    params: arg_types,
                    return_type: Box::new(result_type.clone()),
                    effects: vec![],
                };

                self.unify(&expected_fn, &callee_type, callee.span)?;
                Ok(result_type)
            }

            ExprKind::Lambda { params, body, .. } => {
                let param_types: Vec<InferType> = params
                    .iter()
                    .map(|p| {
                        p.type_ann
                            .as_ref()
                            .map(InferType::from_ast)
                            .unwrap_or_else(|| self.fresh_type())
                    })
                    .collect();

                let old_env = std::mem::take(&mut self.env);
                self.env = TypeEnv::with_parent(old_env);

                for (param, param_type) in params.iter().zip(&param_types) {
                    self.env.define(param.name.clone(), param_type.clone());
                }

                let body_type = self.infer_expr(body)?;

                if let Some(parent) = std::mem::take(&mut self.env).parent {
                    self.env = *parent;
                }

                Ok(InferType::Function {
                    params: param_types,
                    return_type: Box::new(body_type),
                    effects: vec![],
                })
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.infer_expr(condition)?;
                self.unify(
                    &InferType::Concrete("Bool".to_string()),
                    &cond_type,
                    condition.span,
                )?;

                let then_type = self.infer_expr(then_branch)?;

                if let Some(else_br) = else_branch {
                    let else_type = self.infer_expr(else_br)?;
                    self.unify(&then_type, &else_type, else_br.span)?;
                    Ok(then_type)
                } else {
                    Ok(InferType::Unit)
                }
            }

            ExprKind::Match { expr: scrutinee, arms } => {
                let scrutinee_type = self.infer_expr(scrutinee)?;
                let result_type = self.fresh_type();

                for arm in arms {
                    let pattern_type = self.infer_pattern(&arm.pattern)?;
                    self.unify(&scrutinee_type, &pattern_type, arm.pattern.span)?;

                    // TODO: bind pattern variables
                    let arm_type = self.infer_expr(&arm.body)?;
                    self.unify(&result_type, &arm_type, arm.body.span)?;
                }

                Ok(result_type)
            }

            ExprKind::Block { statements, expr } => {
                let old_env = std::mem::take(&mut self.env);
                self.env = TypeEnv::with_parent(old_env);

                for stmt in statements {
                    self.check_stmt(stmt)?;
                }

                let result = if let Some(e) = expr {
                    self.infer_expr(e)?
                } else {
                    InferType::Unit
                };

                if let Some(parent) = std::mem::take(&mut self.env).parent {
                    self.env = *parent;
                }

                Ok(result)
            }

            ExprKind::Array(elements) => {
                let elem_type = self.fresh_type();

                for elem in elements {
                    let t = self.infer_expr(elem)?;
                    self.unify(&elem_type, &t, elem.span)?;
                }

                Ok(InferType::Array(Box::new(elem_type)))
            }

            ExprKind::Tuple(elements) => {
                let types: Vec<InferType> = elements
                    .iter()
                    .map(|e| self.infer_expr(e))
                    .collect::<FiveResult<_>>()?;
                Ok(InferType::Tuple(types))
            }

            ExprKind::Field { object, field: _ } => {
                let _obj_type = self.infer_expr(object)?;
                // TODO: look up field type in struct definition
                Ok(self.fresh_type())
            }

            ExprKind::MethodCall { object, method: _, args: _ } => {
                let _obj_type = self.infer_expr(object)?;
                // TODO: look up method type
                Ok(self.fresh_type())
            }

            ExprKind::Index { object, index } => {
                let obj_type = self.infer_expr(object)?;
                let index_type = self.infer_expr(index)?;

                self.unify(
                    &InferType::Concrete("Int".to_string()),
                    &index_type,
                    index.span,
                )?;

                let elem_type = self.fresh_type();
                self.unify(
                    &InferType::Array(Box::new(elem_type.clone())),
                    &obj_type,
                    object.span,
                )?;

                Ok(elem_type)
            }

            ExprKind::Await(inner) => {
                // For now, just return the inner type
                self.infer_expr(inner)
            }

            ExprKind::Return(inner) => {
                if let Some(e) = inner {
                    self.infer_expr(e)?;
                }
                Ok(InferType::Never)
            }

            ExprKind::Break(_) | ExprKind::Continue => Ok(InferType::Never),

            ExprKind::Range { start, end, .. } => {
                if let Some(s) = start {
                    let t = self.infer_expr(s)?;
                    self.unify(&InferType::Concrete("Int".to_string()), &t, s.span)?;
                }
                if let Some(e) = end {
                    let t = self.infer_expr(e)?;
                    self.unify(&InferType::Concrete("Int".to_string()), &t, e.span)?;
                }
                Ok(InferType::Generic {
                    name: "Range".to_string(),
                    params: vec![InferType::Concrete("Int".to_string())],
                })
            }

            ExprKind::Struct { name, fields } => {
                // TODO: validate fields against struct definition
                for (_, value) in fields {
                    self.infer_expr(value)?;
                }
                Ok(InferType::Concrete(name.clone()))
            }

            ExprKind::Assign { target, value } => {
                let target_type = self.infer_expr(target)?;
                let value_type = self.infer_expr(value)?;
                self.unify(&target_type, &value_type, expr.span)?;
                Ok(value_type)
            }

            ExprKind::CompoundAssign { target, op: _, value } => {
                let target_type = self.infer_expr(target)?;
                let value_type = self.infer_expr(value)?;
                self.unify(&target_type, &value_type, expr.span)?;
                Ok(target_type)
            }
        }
    }

    /// Infer the type of a literal.
    fn infer_literal(&self, lit: &Literal) -> InferType {
        match lit {
            Literal::Int(_) => InferType::Concrete("Int".to_string()),
            Literal::Float(_) => InferType::Concrete("Float".to_string()),
            Literal::String(_) => InferType::Concrete("String".to_string()),
            Literal::Char(_) => InferType::Concrete("Char".to_string()),
            Literal::Bool(_) => InferType::Concrete("Bool".to_string()),
            Literal::Nil => InferType::Unit,
        }
    }

    /// Infer the type of a pattern.
    fn infer_pattern(&mut self, pattern: &Pattern) -> FiveResult<InferType> {
        match &pattern.kind {
            PatternKind::Wildcard => Ok(self.fresh_type()),
            PatternKind::Identifier(_) => Ok(self.fresh_type()),
            PatternKind::Literal(lit) => Ok(self.infer_literal(lit)),
            PatternKind::Constructor { name, args: _ } => {
                // Look up constructor type
                if let Some(ty) = self.env.get(name) {
                    // TODO: unify with args
                    Ok(ty)
                } else {
                    Ok(self.fresh_type())
                }
            }
            PatternKind::Tuple(patterns) => {
                let types: Vec<InferType> = patterns
                    .iter()
                    .map(|p| self.infer_pattern(p))
                    .collect::<FiveResult<_>>()?;
                Ok(InferType::Tuple(types))
            }
        }
    }

    /// Unify two types.
    fn unify(&mut self, expected: &InferType, actual: &InferType, span: Span) -> FiveResult<()> {
        let unifier = Unifier::new(&self.substitutions);
        match unifier.unify(expected, actual) {
            Ok(new_subs) => {
                for (var, ty) in new_subs {
                    self.substitutions.insert(var, ty);
                }
                Ok(())
            }
            Err(msg) => Err(FiveError::type_error(msg, span)),
        }
    }

    /// Resolve a type by applying all substitutions.
    fn resolve(&self, ty: &InferType) -> InferType {
        match ty {
            InferType::Var(var) => {
                if let Some(resolved) = self.substitutions.get(var) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            InferType::Function {
                params,
                return_type,
                effects,
            } => InferType::Function {
                params: params.iter().map(|p| self.resolve(p)).collect(),
                return_type: Box::new(self.resolve(return_type)),
                effects: effects.clone(),
            },
            InferType::Generic { name, params } => InferType::Generic {
                name: name.clone(),
                params: params.iter().map(|p| self.resolve(p)).collect(),
            },
            InferType::Tuple(types) => {
                InferType::Tuple(types.iter().map(|t| self.resolve(t)).collect())
            }
            InferType::Array(inner) => InferType::Array(Box::new(self.resolve(inner))),
            InferType::Reference { inner, mutable } => InferType::Reference {
                inner: Box::new(self.resolve(inner)),
                mutable: *mutable,
            },
            _ => ty.clone(),
        }
    }
}
