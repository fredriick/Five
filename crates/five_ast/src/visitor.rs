//! Visitor pattern for AST traversal.

use crate::{Expr, ExprKind, MatchArm, Pattern, PatternKind, Program, Stmt, StmtKind, Type, TypeKind};

/// A visitor that traverses the AST.
pub trait Visitor: Sized {
    type Output;

    fn visit_program(&mut self, program: &Program) -> Self::Output;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Output;
    fn visit_expr(&mut self, expr: &Expr) -> Self::Output;
    fn visit_type(&mut self, ty: &Type) -> Self::Output;
    fn visit_pattern(&mut self, pattern: &Pattern) -> Self::Output;
}

/// A mutable visitor that can transform the AST.
pub trait MutVisitor: Sized {
    fn visit_program(&mut self, program: &mut Program);
    fn visit_stmt(&mut self, stmt: &mut Stmt);
    fn visit_expr(&mut self, expr: &mut Expr);
    fn visit_type(&mut self, ty: &mut Type);
    fn visit_pattern(&mut self, pattern: &mut Pattern);
}

/// Default walking implementation for immutable visitors.
pub fn walk_program<V: Visitor>(visitor: &mut V, program: &Program) {
    for stmt in &program.statements {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor>(visitor: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Let { value, type_ann, .. } => {
            if let Some(ty) = type_ann {
                visitor.visit_type(ty);
            }
            visitor.visit_expr(value);
        }
        StmtKind::Fn { params, return_type, body, .. } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    visitor.visit_type(ty);
                }
            }
            if let Some(ty) = return_type {
                visitor.visit_type(ty);
            }
            visitor.visit_expr(body);
        }
        StmtKind::Type { definition, .. } => {
            visitor.visit_type(definition);
        }
        StmtKind::Data { variants, .. } => {
            for variant in variants {
                match &variant.fields {
                    crate::VariantFields::Tuple(types) => {
                        for ty in types {
                            visitor.visit_type(ty);
                        }
                    }
                    crate::VariantFields::Named(fields) => {
                        for field in fields {
                            visitor.visit_type(&field.ty);
                        }
                    }
                    crate::VariantFields::Unit => {}
                }
            }
        }
        StmtKind::Struct { fields, .. } => {
            for field in fields {
                visitor.visit_type(&field.ty);
            }
        }
        StmtKind::Expr(expr) => {
            visitor.visit_expr(expr);
        }
        StmtKind::While { condition, body } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(body);
        }
        StmtKind::For { iter, body, .. } => {
            visitor.visit_expr(iter);
            visitor.visit_expr(body);
        }
        StmtKind::Import { .. } => {}
        StmtKind::Export(inner) => {
            visitor.visit_stmt(inner);
        }
    }
}

pub fn walk_expr<V: Visitor>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Literal(_) => {}
        ExprKind::Identifier(_) => {}
        ExprKind::Binary { left, right, .. } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        ExprKind::Unary { expr, .. } => {
            visitor.visit_expr(expr);
        }
        ExprKind::Call { callee, args } => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::MethodCall { object, args, .. } => {
            visitor.visit_expr(object);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::Field { object, .. } => {
            visitor.visit_expr(object);
        }
        ExprKind::Index { object, index } => {
            visitor.visit_expr(object);
            visitor.visit_expr(index);
        }
        ExprKind::Lambda { params, body, .. } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    visitor.visit_type(ty);
                }
            }
            visitor.visit_expr(body);
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            visitor.visit_expr(condition);
            visitor.visit_expr(then_branch);
            if let Some(else_br) = else_branch {
                visitor.visit_expr(else_br);
            }
        }
        ExprKind::Match { expr, arms } => {
            visitor.visit_expr(expr);
            for arm in arms {
                walk_match_arm(visitor, arm);
            }
        }
        ExprKind::Block { statements, expr } => {
            for stmt in statements {
                visitor.visit_stmt(stmt);
            }
            if let Some(e) = expr {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Array(elements) => {
            for elem in elements {
                visitor.visit_expr(elem);
            }
        }
        ExprKind::Tuple(elements) => {
            for elem in elements {
                visitor.visit_expr(elem);
            }
        }
        ExprKind::Await(inner) => {
            visitor.visit_expr(inner);
        }
        ExprKind::Return(inner) => {
            if let Some(e) = inner {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Break(inner) => {
            if let Some(e) = inner {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Continue => {}
        ExprKind::Range { start, end, .. } => {
            if let Some(s) = start {
                visitor.visit_expr(s);
            }
            if let Some(e) = end {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Struct { fields, .. } => {
            for (_, value) in fields {
                visitor.visit_expr(value);
            }
        }
        ExprKind::Assign { target, value } => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        ExprKind::CompoundAssign { target, value, .. } => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
    }
}

pub fn walk_match_arm<V: Visitor>(visitor: &mut V, arm: &MatchArm) {
    visitor.visit_pattern(&arm.pattern);
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard);
    }
    visitor.visit_expr(&arm.body);
}

pub fn walk_pattern<V: Visitor>(visitor: &mut V, pattern: &Pattern) {
    match &pattern.kind {
        PatternKind::Wildcard => {}
        PatternKind::Identifier(_) => {}
        PatternKind::Literal(_) => {}
        PatternKind::Constructor { args, .. } => {
            for arg in args {
                visitor.visit_pattern(arg);
            }
        }
        PatternKind::Tuple(patterns) => {
            for p in patterns {
                visitor.visit_pattern(p);
            }
        }
    }
}

pub fn walk_type<V: Visitor>(visitor: &mut V, ty: &Type) {
    match &ty.kind {
        TypeKind::Named(_) => {}
        TypeKind::Generic { params, .. } => {
            for param in params {
                visitor.visit_type(param);
            }
        }
        TypeKind::Function { params, return_type, .. } => {
            for param in params {
                visitor.visit_type(param);
            }
            visitor.visit_type(return_type);
        }
        TypeKind::Union(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        TypeKind::Reference { inner, .. } => {
            visitor.visit_type(inner);
        }
        TypeKind::Tuple(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        TypeKind::Array(inner) => {
            visitor.visit_type(inner);
        }
        TypeKind::Any => {}
        TypeKind::Infer => {}
        TypeKind::Unit => {}
        TypeKind::Never => {}
    }
}
