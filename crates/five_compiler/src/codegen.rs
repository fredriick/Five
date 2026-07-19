//! Code generation - compiles AST to bytecode.

use five_ast::*;
use five_core::{FiveError, FiveResult};
use five_vm::{Chunk, Instruction, OpCode, VMValue};

/// Local variable information.
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

/// The bytecode compiler.
pub struct Compiler {
    /// Current chunk being compiled
    chunk: Chunk,
    /// Local variables
    locals: Vec<Local>,
    /// Current scope depth
    scope_depth: usize,
    /// Loop context for break/continue
    loop_starts: Vec<usize>,
    loop_breaks: Vec<Vec<usize>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler.
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_starts: Vec::new(),
            loop_breaks: Vec::new(),
        }
    }

    /// Compile a program.
    pub fn compile_program(&mut self, program: &Program) -> FiveResult<Chunk> {
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        Ok(std::mem::take(&mut self.chunk))
    }

    /// Compile a statement.
    fn compile_stmt(&mut self, stmt: &Stmt) -> FiveResult<()> {
        match &stmt.kind {
            StmtKind::Let { name, value, .. } => {
                self.compile_expr(value)?;

                if self.scope_depth > 0 {
                    // Local variable
                    self.add_local(name.clone());
                } else {
                    // Global variable
                    let idx = self.chunk.add_constant(VMValue::String(name.clone()));
                    self.emit(OpCode::DefineGlobal, Some(idx as u32));
                }
            }

            StmtKind::Fn {
                name, params, body, ..
            } => {
                // Compile function body into a separate chunk
                let mut fn_compiler = Compiler::new();
                fn_compiler.scope_depth = 1; // Parameters are at scope depth 1

                // Add parameters as locals
                for param in params {
                    fn_compiler.add_local(param.name.clone());
                }

                fn_compiler.compile_expr(body)?;
                fn_compiler.emit(OpCode::Return, None);

                let fn_chunk = fn_compiler.chunk;

                // Create function value
                let func = VMValue::Function {
                    name: name.clone(),
                    arity: params.len(),
                    chunk: fn_chunk,
                };

                let idx = self.chunk.add_constant(func);
                self.emit(OpCode::Constant, Some(idx as u32));

                if self.scope_depth > 0 {
                    self.add_local(name.clone());
                } else {
                    let name_idx = self.chunk.add_constant(VMValue::String(name.clone()));
                    self.emit(OpCode::DefineGlobal, Some(name_idx as u32));
                }
            }

            StmtKind::Expr(expr) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::Pop, None);
            }

            StmtKind::While { condition, body } => {
                let loop_start = self.chunk.current_offset();
                self.loop_starts.push(loop_start);
                self.loop_breaks.push(Vec::new());

                self.compile_expr(condition)?;

                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit(OpCode::Pop, None); // Pop condition

                self.compile_expr(body)?;
                self.emit(OpCode::Pop, None); // Pop body result

                self.emit_loop(loop_start);

                self.patch_jump(exit_jump);
                self.emit(OpCode::Pop, None); // Pop condition (false path)

                // Patch breaks
                let breaks = self.loop_breaks.pop().unwrap();
                for break_jump in breaks {
                    self.patch_jump(break_jump);
                }
                self.loop_starts.pop();
            }

            StmtKind::For { binding, iter, body } => {
                // Compile iterator expression
                self.compile_expr(iter)?;

                // For now, assume iter is an array
                // TODO: proper iterator protocol

                let loop_start = self.chunk.current_offset();
                self.loop_starts.push(loop_start);
                self.loop_breaks.push(Vec::new());

                // TODO: implement proper for loop with iterator
                // For now, just compile body once (placeholder)
                self.begin_scope();
                self.add_local(binding.clone());
                self.emit(OpCode::Nil, None); // Placeholder for loop var

                self.compile_expr(body)?;
                self.emit(OpCode::Pop, None);

                self.end_scope();

                self.emit(OpCode::Pop, None); // Pop iterator

                let breaks = self.loop_breaks.pop().unwrap();
                for break_jump in breaks {
                    self.patch_jump(break_jump);
                }
                self.loop_starts.pop();
            }

            StmtKind::Data { name, variants, .. } => {
                // Register constructors
                for variant in variants {
                    match &variant.fields {
                        VariantFields::Unit => {
                            // Unit variant is just a constant
                            let idx = self.chunk.add_constant(VMValue::String(format!(
                                "{}::{}",
                                name, variant.name
                            )));
                            self.emit(OpCode::Constant, Some(idx as u32));
                            let name_idx = self
                                .chunk
                                .add_constant(VMValue::String(variant.name.clone()));
                            self.emit(OpCode::DefineGlobal, Some(name_idx as u32));
                        }
                        _ => {
                            // TODO: constructor functions
                        }
                    }
                }
            }

            StmtKind::Struct { .. } | StmtKind::Type { .. } => {
                // Type declarations have no runtime effect
            }

            StmtKind::Import { .. } => {
                // TODO: module system
            }

            StmtKind::Export(inner) => {
                self.compile_stmt(inner)?;
            }
        }

        Ok(())
    }

    /// Compile an expression.
    fn compile_expr(&mut self, expr: &Expr) -> FiveResult<()> {
        match &expr.kind {
            ExprKind::Literal(lit) => {
                match lit {
                    Literal::Int(n) => {
                        let idx = self.chunk.add_constant(VMValue::Int(*n));
                        self.emit(OpCode::Constant, Some(idx as u32));
                    }
                    Literal::Float(n) => {
                        let idx = self.chunk.add_constant(VMValue::Float(*n));
                        self.emit(OpCode::Constant, Some(idx as u32));
                    }
                    Literal::String(s) => {
                        let idx = self.chunk.add_constant(VMValue::String(s.clone()));
                        self.emit(OpCode::Constant, Some(idx as u32));
                    }
                    Literal::Bool(true) => self.emit(OpCode::True, None),
                    Literal::Bool(false) => self.emit(OpCode::False, None),
                    Literal::Nil => self.emit(OpCode::Nil, None),
                    Literal::Char(c) => {
                        let idx = self.chunk.add_constant(VMValue::String(c.to_string()));
                        self.emit(OpCode::Constant, Some(idx as u32));
                    }
                }
            }

            ExprKind::Identifier(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::GetLocal, Some(slot as u32));
                } else {
                    let idx = self.chunk.add_constant(VMValue::String(name.clone()));
                    self.emit(OpCode::GetGlobal, Some(idx as u32));
                }
            }

            ExprKind::Binary { left, op, right } => {
                match op {
                    BinaryOp::And => {
                        self.compile_expr(left)?;
                        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
                        self.emit(OpCode::Pop, None);
                        self.compile_expr(right)?;
                        self.patch_jump(end_jump);
                        return Ok(());
                    }
                    BinaryOp::Or => {
                        self.compile_expr(left)?;
                        let end_jump = self.emit_jump(OpCode::JumpIfTrue);
                        self.emit(OpCode::Pop, None);
                        self.compile_expr(right)?;
                        self.patch_jump(end_jump);
                        return Ok(());
                    }
                    BinaryOp::Pipe => {
                        // left |> right => right(left)
                        self.compile_expr(right)?;
                        self.compile_expr(left)?;
                        self.emit(OpCode::Call, Some(1));
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match op {
                    BinaryOp::Add => self.emit(OpCode::Add, None),
                    BinaryOp::Sub => self.emit(OpCode::Subtract, None),
                    BinaryOp::Mul => self.emit(OpCode::Multiply, None),
                    BinaryOp::Div => self.emit(OpCode::Divide, None),
                    BinaryOp::Mod => self.emit(OpCode::Modulo, None),
                    BinaryOp::Eq => self.emit(OpCode::Equal, None),
                    BinaryOp::Ne => self.emit(OpCode::NotEqual, None),
                    BinaryOp::Lt => self.emit(OpCode::Less, None),
                    BinaryOp::Gt => self.emit(OpCode::Greater, None),
                    BinaryOp::Le => self.emit(OpCode::LessEqual, None),
                    BinaryOp::Ge => self.emit(OpCode::GreaterEqual, None),
                    _ => {}
                }
            }

            ExprKind::Unary { op, expr: inner } => {
                self.compile_expr(inner)?;

                match op {
                    UnaryOp::Neg => self.emit(OpCode::Negate, None),
                    UnaryOp::Not => self.emit(OpCode::Not, None),
                    _ => {} // TODO: references
                }
            }

            ExprKind::Call { callee, args } => {
                self.compile_expr(callee)?;

                for arg in args {
                    self.compile_expr(arg)?;
                }

                self.emit(OpCode::Call, Some(args.len() as u32));
            }

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition)?;

                let then_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.emit(OpCode::Pop, None); // Pop condition

                self.compile_expr(then_branch)?;

                let else_jump = self.emit_jump(OpCode::Jump);

                self.patch_jump(then_jump);
                self.emit(OpCode::Pop, None); // Pop condition (false path)

                if let Some(else_br) = else_branch {
                    self.compile_expr(else_br)?;
                } else {
                    self.emit(OpCode::Nil, None);
                }

                self.patch_jump(else_jump);
            }

            ExprKind::Block { statements, expr } => {
                self.begin_scope();

                for stmt in statements {
                    self.compile_stmt(stmt)?;
                }

                if let Some(e) = expr {
                    self.compile_expr(e)?;
                } else {
                    self.emit(OpCode::Nil, None);
                }

                self.end_scope();
            }

            ExprKind::Array(elements) => {
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                self.emit(OpCode::Array, Some(elements.len() as u32));
            }

            ExprKind::Index { object, index } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.emit(OpCode::Index, None);
            }

            ExprKind::Return(inner) => {
                if let Some(e) = inner {
                    self.compile_expr(e)?;
                } else {
                    self.emit(OpCode::Nil, None);
                }
                self.emit(OpCode::Return, None);
            }

            ExprKind::Break(_) => {
                self.emit(OpCode::Nil, None);
                if !self.loop_breaks.is_empty() {
                    let jump = self.emit_jump(OpCode::Jump);
                    self.loop_breaks.last_mut().unwrap().push(jump);
                }
            }

            ExprKind::Continue => {
                if let Some(&start) = self.loop_starts.last() {
                    self.emit_loop(start);
                }
            }

            ExprKind::Lambda { params, body, .. } => {
                // Compile function body
                let mut fn_compiler = Compiler::new();
                fn_compiler.scope_depth = 1;

                for param in params {
                    fn_compiler.add_local(param.name.clone());
                }

                fn_compiler.compile_expr(body)?;
                fn_compiler.emit(OpCode::Return, None);

                let fn_chunk = fn_compiler.chunk;

                let func = VMValue::Function {
                    name: "<lambda>".to_string(),
                    arity: params.len(),
                    chunk: fn_chunk,
                };

                let idx = self.chunk.add_constant(func);
                self.emit(OpCode::Constant, Some(idx as u32));
            }

            ExprKind::Match { expr: scrutinee, arms } => {
                // Simple match compilation - evaluate scrutinee, test each pattern
                self.compile_expr(scrutinee)?;

                let mut end_jumps = Vec::new();

                for (i, arm) in arms.iter().enumerate() {
                    // For now, only handle simple patterns
                    match &arm.pattern.kind {
                        PatternKind::Wildcard | PatternKind::Identifier(_) => {
                            // Always matches
                            if let PatternKind::Identifier(name) = &arm.pattern.kind {
                                // Bind the variable
                                self.begin_scope();
                                self.add_local(name.clone());
                            }

                            self.compile_expr(&arm.body)?;

                            if let PatternKind::Identifier(_) = &arm.pattern.kind {
                                self.end_scope();
                            }

                            if i < arms.len() - 1 {
                                end_jumps.push(self.emit_jump(OpCode::Jump));
                            }
                        }
                        PatternKind::Literal(lit) => {
                            // Compare with literal
                            match lit {
                                Literal::Int(n) => {
                                    let idx = self.chunk.add_constant(VMValue::Int(*n));
                                    self.emit(OpCode::Constant, Some(idx as u32));
                                }
                                Literal::Bool(true) => self.emit(OpCode::True, None),
                                Literal::Bool(false) => self.emit(OpCode::False, None),
                                _ => {}
                            }
                            self.emit(OpCode::Equal, None);

                            let skip_jump = self.emit_jump(OpCode::JumpIfFalse);
                            self.emit(OpCode::Pop, None);

                            self.compile_expr(&arm.body)?;

                            end_jumps.push(self.emit_jump(OpCode::Jump));

                            self.patch_jump(skip_jump);
                            self.emit(OpCode::Pop, None);
                        }
                        _ => {
                            // TODO: more complex patterns
                            return Err(FiveError::runtime(
                                "Complex patterns not yet supported in compiler",
                                arm.pattern.span,
                            ));
                        }
                    }
                }

                // Patch all end jumps
                for jump in end_jumps {
                    self.patch_jump(jump);
                }

                // Pop the scrutinee
                self.emit(OpCode::Pop, None);
            }

            ExprKind::Assign { target, value } => {
                self.compile_expr(value)?;

                match &target.kind {
                    ExprKind::Identifier(name) => {
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::SetLocal, Some(slot as u32));
                        } else {
                            let idx = self.chunk.add_constant(VMValue::String(name.clone()));
                            self.emit(OpCode::SetGlobal, Some(idx as u32));
                        }
                    }
                    _ => {
                        return Err(FiveError::runtime(
                            "Complex assignment targets not yet supported in compiler",
                            target.span,
                        ));
                    }
                }
            }

            ExprKind::CompoundAssign { target, op, value } => {
                // Load current value
                match &target.kind {
                    ExprKind::Identifier(name) => {
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::GetLocal, Some(slot as u32));
                        } else {
                            let idx = self.chunk.add_constant(VMValue::String(name.clone()));
                            self.emit(OpCode::GetGlobal, Some(idx as u32));
                        }
                    }
                    _ => {
                        return Err(FiveError::runtime(
                            "Complex assignment targets not yet supported in compiler",
                            target.span,
                        ));
                    }
                }

                // Compile the value and apply the operation
                self.compile_expr(value)?;

                match op {
                    BinaryOp::Add => self.emit(OpCode::Add, None),
                    BinaryOp::Sub => self.emit(OpCode::Subtract, None),
                    BinaryOp::Mul => self.emit(OpCode::Multiply, None),
                    BinaryOp::Div => self.emit(OpCode::Divide, None),
                    _ => {}
                }

                // Store back
                match &target.kind {
                    ExprKind::Identifier(name) => {
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::SetLocal, Some(slot as u32));
                        } else {
                            let idx = self.chunk.add_constant(VMValue::String(name.clone()));
                            self.emit(OpCode::SetGlobal, Some(idx as u32));
                        }
                    }
                    _ => {}
                }
            }

            // TODO: implement remaining expression types
            ExprKind::MethodCall { .. }
            | ExprKind::Field { .. }
            | ExprKind::Tuple(_)
            | ExprKind::Await(_)
            | ExprKind::Range { .. }
            | ExprKind::Struct { .. } => {
                // Placeholder - emit nil for now
                self.emit(OpCode::Nil, None);
            }
        }

        Ok(())
    }

    // Helper methods

    fn emit(&mut self, opcode: OpCode, operand: Option<u32>) {
        self.chunk
            .write(Instruction::new(opcode, operand), 1);
    }

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit(opcode, Some(0)); // Placeholder
        self.chunk.current_offset() - 1
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit(OpCode::Jump, Some(loop_start as u32));
    }

    fn patch_jump(&mut self, offset: usize) {
        self.chunk.patch_jump(offset);
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while !self.locals.is_empty()
            && self.locals.last().unwrap().depth > self.scope_depth
        {
            self.locals.pop();
            self.emit(OpCode::Pop, None);
        }
    }

    fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_vm::{Instruction, OpCode};

    #[test]
    fn test_compile_constant() {
        let mut compiler = Compiler::new();
        let program = five_parser::Parser::parse("42").unwrap();
        let chunk = compiler.compile_program(&program).unwrap();

        // Should have: CONSTANT 42, POP
        assert!(chunk.constants.contains(&VMValue::Int(42)));
    }

    #[test]
    fn test_compile_binary() {
        let mut compiler = Compiler::new();
        let program = five_parser::Parser::parse("1 + 2").unwrap();
        let chunk = compiler.compile_program(&program).unwrap();

        // Should have constants 1 and 2
        assert!(chunk.constants.contains(&VMValue::Int(1)));
        assert!(chunk.constants.contains(&VMValue::Int(2)));

        // Should have ADD instruction
        let has_add = chunk.code.iter().any(|i| i.opcode == OpCode::Add);
        assert!(has_add);
    }
}
