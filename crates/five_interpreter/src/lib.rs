//! Five Interpreter - Tree-walking interpreter for the Five programming language.

mod builtins;
mod env;
mod value;

pub use env::Environment;
pub use value::Value;

use five_ast::*;
use five_core::{FiveError, FiveResult, Span};
use five_parser::Parser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The Five interpreter.
pub struct Interpreter {
    /// The global environment.
    env: Rc<RefCell<Environment>>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new interpreter.
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));
        builtins::register_builtins(&env);
        Self { env }
    }

    /// Run source code.
    pub fn run(&mut self, source: &str) -> FiveResult<Value> {
        let program = Parser::parse(source)?;
        self.eval_program(&program)
    }

    /// Evaluate a program.
    pub fn eval_program(&mut self, program: &Program) -> FiveResult<Value> {
        let mut result = Value::Nil;

        for stmt in &program.statements {
            result = self.eval_stmt(stmt)?;

            // Check for early return
            if let Value::Return(v) = result {
                return Ok(*v);
            }
        }

        Ok(result)
    }

    /// Evaluate a statement.
    pub fn eval_stmt(&mut self, stmt: &Stmt) -> FiveResult<Value> {
        match &stmt.kind {
            StmtKind::Let { name, value, mutable, .. } => {
                let val = self.eval_expr(value)?;
                self.env.borrow_mut().define_mut(name.clone(), val, *mutable);
                Ok(Value::Nil)
            }

            StmtKind::Fn {
                name,
                params,
                body,
                ..
            } => {
                let func = Value::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    env: Rc::clone(&self.env),
                };
                self.env.borrow_mut().define(name.clone(), func);
                Ok(Value::Nil)
            }

            StmtKind::Data { name, variants, .. } => {
                // Register constructors for each variant
                for variant in variants {
                    let constructor_name = variant.name.clone();
                    match &variant.fields {
                        VariantFields::Unit => {
                            // Unit constructor is just a value
                            self.env.borrow_mut().define(
                                constructor_name.clone(),
                                Value::DataVariant {
                                    type_name: name.clone(),
                                    variant: constructor_name,
                                    values: vec![],
                                },
                            );
                        }
                        VariantFields::Tuple(types) => {
                            // Tuple constructor is a function
                            let arity = types.len();
                            self.env.borrow_mut().define(
                                constructor_name.clone(),
                                Value::DataConstructor {
                                    type_name: name.clone(),
                                    variant: constructor_name,
                                    arity,
                                },
                            );
                        }
                        VariantFields::Named(fields) => {
                            // Named constructor is a function
                            let arity = fields.len();
                            self.env.borrow_mut().define(
                                constructor_name.clone(),
                                Value::DataConstructor {
                                    type_name: name.clone(),
                                    variant: constructor_name,
                                    arity,
                                },
                            );
                        }
                    }
                }
                Ok(Value::Nil)
            }

            StmtKind::Struct { name, fields, .. } => {
                // Register a struct constructor
                let field_names: Vec<String> =
                    fields.iter().map(|f| f.name.clone()).collect();
                self.env.borrow_mut().define(
                    name.clone(),
                    Value::StructConstructor {
                        name: name.clone(),
                        fields: field_names,
                    },
                );
                Ok(Value::Nil)
            }

            StmtKind::Type { .. } => {
                // Type aliases don't have runtime representation
                Ok(Value::Nil)
            }

            StmtKind::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    let result = self.eval_expr(body)?;
                    if let Value::Break(v) = result {
                        return Ok(v.map(|b| *b).unwrap_or(Value::Nil));
                    }
                    if matches!(result, Value::Continue) {
                        continue;
                    }
                    if matches!(result, Value::Return(_)) {
                        return Ok(result);
                    }
                }
                Ok(Value::Nil)
            }

            StmtKind::For { binding, iter, body } => {
                let iterable = self.eval_expr(iter)?;
                let items = match iterable {
                    Value::Array(items) => items,
                    Value::Range { start, end, inclusive } => {
                        let end_val = if inclusive { end + 1 } else { end };
                        (start..end_val).map(Value::Int).collect()
                    }
                    _ => {
                        return Err(FiveError::runtime(
                            "Cannot iterate over non-iterable value",
                            iter.span,
                        ))
                    }
                };

                for item in items {
                    let new_env = Environment::with_parent(Rc::clone(&self.env));
                    let new_env = Rc::new(RefCell::new(new_env));
                    new_env.borrow_mut().define(binding.clone(), item);

                    let old_env = std::mem::replace(&mut self.env, new_env);
                    let result = self.eval_expr(body);
                    self.env = old_env;

                    let result = result?;
                    if let Value::Break(v) = result {
                        return Ok(v.map(|b| *b).unwrap_or(Value::Nil));
                    }
                    if matches!(result, Value::Continue) {
                        continue;
                    }
                    if matches!(result, Value::Return(_)) {
                        return Ok(result);
                    }
                }
                Ok(Value::Nil)
            }

            StmtKind::Import { .. } => {
                // Imports are not yet implemented
                Ok(Value::Nil)
            }

            StmtKind::Export(inner) => {
                // Exports just evaluate the inner statement
                self.eval_stmt(inner)
            }

            StmtKind::Expr(expr) => self.eval_expr(expr),
        }
    }

    /// Evaluate an expression.
    pub fn eval_expr(&mut self, expr: &Expr) -> FiveResult<Value> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.eval_literal(lit)),

            ExprKind::Identifier(name) => {
                self.env.borrow().get(name).ok_or_else(|| {
                    FiveError::runtime(format!("Undefined variable: {}", name), expr.span)
                })
            }

            ExprKind::Binary { left, op, right } => {
                self.eval_binary(left, *op, right, expr.span)
            }

            ExprKind::Unary { op, expr: inner } => {
                self.eval_unary(*op, inner, expr.span)
            }

            ExprKind::Call { callee, args } => {
                self.eval_call(callee, args, expr.span)
            }

            ExprKind::MethodCall { object, method, args } => {
                self.eval_method_call(object, method, args, expr.span)
            }

            ExprKind::Field { object, field } => {
                let obj = self.eval_expr(object)?;
                match obj {
                    Value::Struct { fields, .. } => {
                        fields.get(field).cloned().ok_or_else(|| {
                            FiveError::runtime(
                                format!("No field '{}' on struct", field),
                                expr.span,
                            )
                        })
                    }
                    _ => Err(FiveError::runtime(
                        "Cannot access field on non-struct value",
                        expr.span,
                    )),
                }
            }

            ExprKind::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;

                match (&obj, &idx) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let i = *i as usize;
                        arr.get(i).cloned().ok_or_else(|| {
                            FiveError::runtime(
                                format!("Index {} out of bounds", i),
                                expr.span,
                            )
                        })
                    }
                    (Value::String(s), Value::Int(i)) => {
                        let i = *i as usize;
                        s.chars().nth(i).map(|c| Value::Char(c)).ok_or_else(|| {
                            FiveError::runtime(
                                format!("Index {} out of bounds", i),
                                expr.span,
                            )
                        })
                    }
                    _ => Err(FiveError::runtime(
                        "Cannot index with given types",
                        expr.span,
                    )),
                }
            }

            ExprKind::Lambda {
                params, body, ..
            } => {
                Ok(Value::Function {
                    name: "<lambda>".to_string(),
                    params: params.clone(),
                    body: (**body).clone(),
                    env: Rc::clone(&self.env),
                })
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.eval_expr(condition)?;
                if cond.is_truthy() {
                    self.eval_expr(then_branch)
                } else if let Some(else_br) = else_branch {
                    self.eval_expr(else_br)
                } else {
                    Ok(Value::Nil)
                }
            }

            ExprKind::Match { expr: scrutinee, arms } => {
                let value = self.eval_expr(scrutinee)?;
                self.eval_match(&value, arms, expr.span)
            }

            ExprKind::Block { statements, expr: final_expr } => {
                let new_env = Environment::with_parent(Rc::clone(&self.env));
                let new_env = Rc::new(RefCell::new(new_env));
                let old_env = std::mem::replace(&mut self.env, new_env);

                let mut result = Value::Nil;
                for stmt in statements {
                    result = self.eval_stmt(stmt)?;
                    if matches!(result, Value::Return(_) | Value::Break(_) | Value::Continue) {
                        self.env = old_env;
                        return Ok(result);
                    }
                }

                if let Some(final_e) = final_expr {
                    result = self.eval_expr(final_e)?;
                }

                self.env = old_env;
                Ok(result)
            }

            ExprKind::Array(elements) => {
                let values: FiveResult<Vec<Value>> =
                    elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(values?))
            }

            ExprKind::Tuple(elements) => {
                let values: FiveResult<Vec<Value>> =
                    elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Tuple(values?))
            }

            ExprKind::Await(inner) => {
                // For now, just evaluate the inner expression
                // Real async would require a runtime
                self.eval_expr(inner)
            }

            ExprKind::Return(inner) => {
                let value = if let Some(e) = inner {
                    self.eval_expr(e)?
                } else {
                    Value::Nil
                };
                Ok(Value::Return(Box::new(value)))
            }

            ExprKind::Break(inner) => {
                let value = if let Some(e) = inner {
                    Some(Box::new(self.eval_expr(e)?))
                } else {
                    None
                };
                Ok(Value::Break(value))
            }

            ExprKind::Continue => Ok(Value::Continue),

            ExprKind::Range { start, end, inclusive } => {
                let start_val = if let Some(s) = start {
                    match self.eval_expr(s)? {
                        Value::Int(i) => i,
                        _ => {
                            return Err(FiveError::runtime(
                                "Range bounds must be integers",
                                expr.span,
                            ))
                        }
                    }
                } else {
                    0
                };

                let end_val = if let Some(e) = end {
                    match self.eval_expr(e)? {
                        Value::Int(i) => i,
                        _ => {
                            return Err(FiveError::runtime(
                                "Range bounds must be integers",
                                expr.span,
                            ))
                        }
                    }
                } else {
                    i64::MAX
                };

                Ok(Value::Range {
                    start: start_val,
                    end: end_val,
                    inclusive: *inclusive,
                })
            }

            ExprKind::Struct { name, fields } => {
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    field_values.insert(field_name.clone(), self.eval_expr(field_expr)?);
                }
                Ok(Value::Struct {
                    name: name.clone(),
                    fields: field_values,
                })
            }

            ExprKind::Assign { target, value } => {
                let val = self.eval_expr(value)?;
                self.assign_to_target(target, val.clone(), expr.span)?;
                Ok(val)
            }

            ExprKind::CompoundAssign { target, op, value } => {
                let current = self.eval_expr(target)?;
                let rhs = self.eval_expr(value)?;
                let new_val = self.eval_binary_values(&current, *op, &rhs, expr.span)?;
                self.assign_to_target(target, new_val.clone(), expr.span)?;
                Ok(new_val)
            }
        }
    }

    /// Assign a value to an assignment target.
    fn assign_to_target(&mut self, target: &Expr, value: Value, span: Span) -> FiveResult<()> {
        match &target.kind {
            ExprKind::Identifier(name) => {
                match self.env.borrow_mut().set(name, value) {
                    Ok(true) => Ok(()),
                    Ok(false) => Err(FiveError::runtime(
                        format!("Cannot assign to undefined variable: {}", name),
                        span,
                    )),
                    Err(msg) => Err(FiveError::runtime(
                        format!("{}: {}", msg, name),
                        span,
                    )),
                }
            }
            ExprKind::Index { object, index } => {
                let idx = self.eval_expr(index)?;
                self.assign_to_index(object, idx, value, span)
            }
            ExprKind::Field { object, field } => {
                self.assign_to_field(object, field, value, span)
            }
            _ => Err(FiveError::runtime("Invalid assignment target", span)),
        }
    }

    /// Assign to an array index.
    fn assign_to_index(&mut self, object: &Expr, index: Value, value: Value, span: Span) -> FiveResult<()> {
        // Get the variable name being indexed
        if let ExprKind::Identifier(name) = &object.kind {
            let obj = self.env.borrow().get(name).ok_or_else(|| {
                FiveError::runtime(format!("Undefined variable: {}", name), span)
            })?;

            match (obj, &index) {
                (Value::Array(mut arr), Value::Int(i)) => {
                    let i = *i as usize;
                    if i >= arr.len() {
                        return Err(FiveError::runtime(
                            format!("Index {} out of bounds for array of length {}", i, arr.len()),
                            span,
                        ));
                    }
                    arr[i] = value;
                    // Use set_unchecked for array mutation (arrays are mutable by reference)
                    self.env.borrow_mut().set_unchecked(name, Value::Array(arr));
                    Ok(())
                }
                _ => Err(FiveError::runtime(
                    "Cannot index-assign to non-array or with non-integer index",
                    span,
                )),
            }
        } else {
            Err(FiveError::runtime(
                "Cannot index-assign to complex expression",
                span,
            ))
        }
    }

    /// Assign to a struct field.
    fn assign_to_field(&mut self, object: &Expr, field: &str, value: Value, span: Span) -> FiveResult<()> {
        // Get the variable name being field-accessed
        if let ExprKind::Identifier(name) = &object.kind {
            let obj = self.env.borrow().get(name).ok_or_else(|| {
                FiveError::runtime(format!("Undefined variable: {}", name), span)
            })?;

            match obj {
                Value::Struct { name: struct_name, mut fields } => {
                    if !fields.contains_key(field) {
                        return Err(FiveError::runtime(
                            format!("No field '{}' on struct '{}'", field, struct_name),
                            span,
                        ));
                    }
                    fields.insert(field.to_string(), value);
                    // Use set_unchecked for struct field mutation
                    self.env.borrow_mut().set_unchecked(name, Value::Struct { name: struct_name, fields });
                    Ok(())
                }
                _ => Err(FiveError::runtime(
                    "Cannot field-assign to non-struct",
                    span,
                )),
            }
        } else {
            Err(FiveError::runtime(
                "Cannot field-assign to complex expression",
                span,
            ))
        }
    }

    /// Evaluate a binary operation on values directly.
    fn eval_binary_values(&self, left: &Value, op: BinaryOp, right: &Value, span: Span) -> FiveResult<Value> {
        match (op, left, right) {
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Add, Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(FiveError::runtime("Division by zero", span))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinaryOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            _ => Err(FiveError::runtime(
                format!("Invalid operands for {:?}", op),
                span,
            )),
        }
    }

    /// Evaluate a literal.
    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(n) => Value::Float(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::Char(*c),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Nil => Value::Nil,
        }
    }

    /// Evaluate a binary operation.
    fn eval_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> FiveResult<Value> {
        // Short-circuit evaluation for logical operators
        if op == BinaryOp::And {
            let left_val = self.eval_expr(left)?;
            if !left_val.is_truthy() {
                return Ok(Value::Bool(false));
            }
            let right_val = self.eval_expr(right)?;
            return Ok(Value::Bool(right_val.is_truthy()));
        }

        if op == BinaryOp::Or {
            let left_val = self.eval_expr(left)?;
            if left_val.is_truthy() {
                return Ok(Value::Bool(true));
            }
            let right_val = self.eval_expr(right)?;
            return Ok(Value::Bool(right_val.is_truthy()));
        }

        // Pipe operator
        if op == BinaryOp::Pipe {
            let arg = self.eval_expr(left)?;
            let func = self.eval_expr(right)?;
            return self.call_value(&func, vec![arg], span);
        }

        let left_val = self.eval_expr(left)?;
        let right_val = self.eval_expr(right)?;

        match (op, &left_val, &right_val) {
            // Arithmetic
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Add, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (BinaryOp::Add, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (BinaryOp::Add, Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }
            (BinaryOp::Add, Value::String(a), b) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }

            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOp::Sub, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (BinaryOp::Sub, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),

            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOp::Mul, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (BinaryOp::Mul, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),

            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(FiveError::runtime("Division by zero", span))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinaryOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (BinaryOp::Div, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (BinaryOp::Div, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),

            (BinaryOp::Mod, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(FiveError::runtime("Modulo by zero", span))
                } else {
                    Ok(Value::Int(a % b))
                }
            }

            // Comparison
            (BinaryOp::Eq, _, _) => Ok(Value::Bool(left_val == right_val)),
            (BinaryOp::Ne, _, _) => Ok(Value::Bool(left_val != right_val)),

            (BinaryOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (BinaryOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

            _ => Err(FiveError::runtime(
                format!(
                    "Invalid operands for {:?}: {} and {}",
                    op,
                    left_val.type_name(),
                    right_val.type_name()
                ),
                span,
            )),
        }
    }

    /// Evaluate a unary operation.
    fn eval_unary(&mut self, op: UnaryOp, expr: &Expr, span: Span) -> FiveResult<Value> {
        let value = self.eval_expr(expr)?;

        match (op, &value) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnaryOp::Not, _) => Ok(Value::Bool(!value.is_truthy())),
            (UnaryOp::Ref, _) => {
                // For now, references just wrap the value
                Ok(Value::Ref(Box::new(value)))
            }
            (UnaryOp::MutRef, _) => Ok(Value::MutRef(Box::new(value))),
            (UnaryOp::Deref, Value::Ref(inner)) => Ok(*inner.clone()),
            (UnaryOp::Deref, Value::MutRef(inner)) => Ok(*inner.clone()),
            _ => Err(FiveError::runtime(
                format!("Invalid unary operation {:?} on {}", op, value.type_name()),
                span,
            )),
        }
    }

    /// Evaluate a function call.
    fn eval_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> FiveResult<Value> {
        let func = self.eval_expr(callee)?;
        let arg_values: FiveResult<Vec<Value>> =
            args.iter().map(|a| self.eval_expr(a)).collect();
        self.call_value(&func, arg_values?, span)
    }

    /// Call a value (function, constructor, etc.).
    fn call_value(&mut self, func: &Value, args: Vec<Value>, span: Span) -> FiveResult<Value> {
        match func {
            Value::Function { params, body, env, .. } => {
                if args.len() != params.len() {
                    return Err(FiveError::runtime(
                        format!(
                            "Expected {} arguments, got {}",
                            params.len(),
                            args.len()
                        ),
                        span,
                    ));
                }

                // Create new environment with parameters
                let new_env = Environment::with_parent(Rc::clone(env));
                let new_env = Rc::new(RefCell::new(new_env));

                for (param, arg) in params.iter().zip(args) {
                    new_env.borrow_mut().define(param.name.clone(), arg);
                }

                let old_env = std::mem::replace(&mut self.env, new_env);
                let result = self.eval_expr(body);
                self.env = old_env;

                match result? {
                    Value::Return(v) => Ok(*v),
                    other => Ok(other),
                }
            }

            Value::BuiltinFunction { func, .. } => func(args, span),

            Value::DataConstructor {
                type_name,
                variant,
                arity,
            } => {
                if args.len() != *arity {
                    return Err(FiveError::runtime(
                        format!(
                            "Constructor {} expects {} arguments, got {}",
                            variant, arity, args.len()
                        ),
                        span,
                    ));
                }
                Ok(Value::DataVariant {
                    type_name: type_name.clone(),
                    variant: variant.clone(),
                    values: args,
                })
            }

            Value::StructConstructor { name, fields } => {
                if args.len() != fields.len() {
                    return Err(FiveError::runtime(
                        format!(
                            "Struct {} expects {} fields, got {}",
                            name,
                            fields.len(),
                            args.len()
                        ),
                        span,
                    ));
                }
                let mut field_map = HashMap::new();
                for (field_name, value) in fields.iter().zip(args) {
                    field_map.insert(field_name.clone(), value);
                }
                Ok(Value::Struct {
                    name: name.clone(),
                    fields: field_map,
                })
            }

            _ => Err(FiveError::runtime(
                format!("Cannot call {}", func.type_name()),
                span,
            )),
        }
    }

    /// Evaluate a method call.
    fn eval_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        span: Span,
    ) -> FiveResult<Value> {
        let obj = self.eval_expr(object)?;
        let mut arg_values: Vec<Value> =
            args.iter().map(|a| self.eval_expr(a)).collect::<FiveResult<_>>()?;

        // Built-in methods
        match (&obj, method) {
            (Value::Array(arr), "len") => {
                return Ok(Value::Int(arr.len() as i64));
            }
            (Value::Array(arr), "push") if args.len() == 1 => {
                let mut new_arr = arr.clone();
                new_arr.push(arg_values.remove(0));
                return Ok(Value::Array(new_arr));
            }
            (Value::Array(arr), "map") if args.len() == 1 => {
                let func = arg_values.remove(0);
                let mut results = Vec::new();
                for item in arr {
                    results.push(self.call_value(&func, vec![item.clone()], span)?);
                }
                return Ok(Value::Array(results));
            }
            (Value::Array(arr), "filter") if args.len() == 1 => {
                let func = arg_values.remove(0);
                let mut results = Vec::new();
                for item in arr {
                    let keep = self.call_value(&func, vec![item.clone()], span)?;
                    if keep.is_truthy() {
                        results.push(item.clone());
                    }
                }
                return Ok(Value::Array(results));
            }
            (Value::Array(arr), "fold") if args.len() == 2 => {
                let init = arg_values.remove(0);
                let func = arg_values.remove(0);
                let mut acc = init;
                for item in arr {
                    acc = self.call_value(&func, vec![acc, item.clone()], span)?;
                }
                return Ok(acc);
            }
            (Value::String(s), "len") => {
                return Ok(Value::Int(s.len() as i64));
            }
            (Value::String(s), "chars") => {
                return Ok(Value::Array(s.chars().map(Value::Char).collect()));
            }
            (Value::String(s), "split") if args.len() == 1 => {
                if let Value::String(sep) = &arg_values[0] {
                    return Ok(Value::Array(
                        s.split(sep.as_str())
                            .map(|part| Value::String(part.to_string()))
                            .collect(),
                    ));
                }
            }
            (Value::String(s), "trim") => {
                return Ok(Value::String(s.trim().to_string()));
            }
            (Value::String(s), "trim_start") => {
                return Ok(Value::String(s.trim_start().to_string()));
            }
            (Value::String(s), "trim_end") => {
                return Ok(Value::String(s.trim_end().to_string()));
            }
            (Value::String(s), "uppercase") => {
                return Ok(Value::String(s.to_uppercase()));
            }
            (Value::String(s), "lowercase") => {
                return Ok(Value::String(s.to_lowercase()));
            }
            (Value::String(s), "contains") if args.len() == 1 => {
                if let Value::String(sub) = &arg_values[0] {
                    return Ok(Value::Bool(s.contains(sub.as_str())));
                }
            }
            (Value::String(s), "starts_with") if args.len() == 1 => {
                if let Value::String(prefix) = &arg_values[0] {
                    return Ok(Value::Bool(s.starts_with(prefix.as_str())));
                }
            }
            (Value::String(s), "ends_with") if args.len() == 1 => {
                if let Value::String(suffix) = &arg_values[0] {
                    return Ok(Value::Bool(s.ends_with(suffix.as_str())));
                }
            }
            (Value::String(s), "replace") if args.len() == 2 => {
                if let (Value::String(from), Value::String(to)) = (&arg_values[0], &arg_values[1]) {
                    return Ok(Value::String(s.replace(from.as_str(), to.as_str())));
                }
            }
            (Value::String(s), "substring") if args.len() == 2 => {
                if let (Value::Int(start), Value::Int(end)) = (&arg_values[0], &arg_values[1]) {
                    let start = *start as usize;
                    let end = *end as usize;
                    if start <= end && end <= s.len() {
                        return Ok(Value::String(s[start..end].to_string()));
                    }
                    return Err(FiveError::runtime("substring indices out of bounds", span));
                }
            }
            (Value::String(s), "repeat") if args.len() == 1 => {
                if let Value::Int(n) = &arg_values[0] {
                    return Ok(Value::String(s.repeat(*n as usize)));
                }
            }
            (Value::String(s), "reverse") => {
                return Ok(Value::String(s.chars().rev().collect()));
            }
            (Value::String(s), "is_empty") => {
                return Ok(Value::Bool(s.is_empty()));
            }
            (Value::String(s), "lines") => {
                return Ok(Value::Array(
                    s.lines()
                        .map(|line| Value::String(line.to_string()))
                        .collect(),
                ));
            }
            (Value::String(s), "join") if args.len() == 1 => {
                // Used as separator.join(array)
                if let Value::Array(arr) = &arg_values[0] {
                    let strings: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                    return Ok(Value::String(strings.join(s)));
                }
            }
            (Value::Array(arr), "join") if args.len() == 1 => {
                // Used as array.join(separator)
                if let Value::String(sep) = &arg_values[0] {
                    let strings: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                    return Ok(Value::String(strings.join(sep)));
                }
            }
            (Value::Array(arr), "reverse") => {
                let mut new_arr = arr.clone();
                new_arr.reverse();
                return Ok(Value::Array(new_arr));
            }
            (Value::Array(arr), "contains") if args.len() == 1 => {
                return Ok(Value::Bool(arr.contains(&arg_values[0])));
            }
            (Value::Array(arr), "is_empty") => {
                return Ok(Value::Bool(arr.is_empty()));
            }
            (Value::Array(arr), "first") => {
                return Ok(arr.first().cloned().unwrap_or(Value::Nil));
            }
            (Value::Array(arr), "last") => {
                return Ok(arr.last().cloned().unwrap_or(Value::Nil));
            }
            (Value::Array(arr), "take") if args.len() == 1 => {
                if let Value::Int(n) = &arg_values[0] {
                    let n = *n as usize;
                    return Ok(Value::Array(arr.iter().take(n).cloned().collect()));
                }
            }
            (Value::Array(arr), "skip") if args.len() == 1 => {
                if let Value::Int(n) = &arg_values[0] {
                    let n = *n as usize;
                    return Ok(Value::Array(arr.iter().skip(n).cloned().collect()));
                }
            }
            _ => {}
        }

        Err(FiveError::runtime(
            format!("No method '{}' on {}", method, obj.type_name()),
            span,
        ))
    }

    /// Evaluate a match expression.
    fn eval_match(
        &mut self,
        value: &Value,
        arms: &[MatchArm],
        span: Span,
    ) -> FiveResult<Value> {
        for arm in arms {
            if let Some(bindings) = self.match_pattern(&arm.pattern, value)? {
                // Check guard if present
                if let Some(guard) = &arm.guard {
                    let new_env = Environment::with_parent(Rc::clone(&self.env));
                    let new_env = Rc::new(RefCell::new(new_env));
                    for (name, val) in &bindings {
                        new_env.borrow_mut().define(name.clone(), val.clone());
                    }
                    let old_env = std::mem::replace(&mut self.env, new_env);
                    let guard_result = self.eval_expr(guard);
                    self.env = old_env;

                    if !guard_result?.is_truthy() {
                        continue;
                    }
                }

                // Evaluate the arm body with bindings
                let new_env = Environment::with_parent(Rc::clone(&self.env));
                let new_env = Rc::new(RefCell::new(new_env));
                for (name, val) in bindings {
                    new_env.borrow_mut().define(name, val);
                }
                let old_env = std::mem::replace(&mut self.env, new_env);
                let result = self.eval_expr(&arm.body);
                self.env = old_env;
                return result;
            }
        }

        Err(FiveError::runtime("Non-exhaustive match", span))
    }

    /// Try to match a pattern against a value, returning bindings if successful.
    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> FiveResult<Option<Vec<(String, Value)>>> {
        match &pattern.kind {
            PatternKind::Wildcard => Ok(Some(vec![])),

            PatternKind::Identifier(name) => {
                Ok(Some(vec![(name.clone(), value.clone())]))
            }

            PatternKind::Literal(lit) => {
                let lit_value = self.eval_literal(lit);
                if &lit_value == value {
                    Ok(Some(vec![]))
                } else {
                    Ok(None)
                }
            }

            PatternKind::Constructor { name, args } => {
                match value {
                    Value::DataVariant { variant, values, .. } => {
                        if name != variant || args.len() != values.len() {
                            return Ok(None);
                        }

                        let mut bindings = vec![];
                        for (pat, val) in args.iter().zip(values) {
                            match self.match_pattern(pat, val)? {
                                Some(mut b) => bindings.append(&mut b),
                                None => return Ok(None),
                            }
                        }
                        Ok(Some(bindings))
                    }
                    _ => Ok(None),
                }
            }

            PatternKind::Tuple(patterns) => {
                match value {
                    Value::Tuple(values) => {
                        if patterns.len() != values.len() {
                            return Ok(None);
                        }

                        let mut bindings = vec![];
                        for (pat, val) in patterns.iter().zip(values) {
                            match self.match_pattern(pat, val)? {
                                Some(mut b) => bindings.append(&mut b),
                                None => return Ok(None),
                            }
                        }
                        Ok(Some(bindings))
                    }
                    _ => Ok(None),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.run("1 + 2").unwrap(), Value::Int(3));
        assert_eq!(interp.run("10 - 3").unwrap(), Value::Int(7));
        assert_eq!(interp.run("4 * 5").unwrap(), Value::Int(20));
        assert_eq!(interp.run("20 / 4").unwrap(), Value::Int(5));
    }

    #[test]
    fn test_variables() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.run("let x = 42; x").unwrap(),
            Value::Int(42)
        );
    }

    #[test]
    fn test_functions() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.run("fn add(a, b) { a + b }\nadd(2, 3)").unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_pipe() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.run("fn double(x) { x * 2 }\n5 |> double").unwrap(),
            Value::Int(10)
        );
    }

    #[test]
    fn test_if_expr() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.run("if true { 1 } else { 2 }").unwrap(),
            Value::Int(1)
        );
        assert_eq!(
            interp.run("if false { 1 } else { 2 }").unwrap(),
            Value::Int(2)
        );
    }

    #[test]
    fn test_match() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.run(r#"
                data Option<T> { Some(T), None }
                let x = Some(42)
                match x {
                    Some(n) => n,
                    None => 0
                }
            "#).unwrap(),
            Value::Int(42)
        );
    }
}
