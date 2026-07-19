//! Five VM - Stack-based virtual machine for the Five programming language.

mod bytecode;
mod chunk;

pub use bytecode::{Instruction, OpCode};
pub use chunk::Chunk;

use five_core::{FiveError, FiveResult, Span};
use std::collections::HashMap;

/// A runtime value in the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum VMValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<VMValue>),
    Function {
        name: String,
        arity: usize,
        chunk: Chunk,
    },
    NativeFunction {
        name: String,
        arity: usize,
        func: fn(&[VMValue]) -> FiveResult<VMValue>,
    },
}

impl VMValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(b) => *b,
            Self::Int(n) => *n != 0,
            Self::Float(n) => *n != 0.0,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Function { .. } => "function",
            Self::NativeFunction { .. } => "native_function",
        }
    }
}

impl std::fmt::Display for VMValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Self::Function { name, .. } => write!(f, "<fn {}>", name),
            Self::NativeFunction { name, .. } => write!(f, "<native {}>", name),
        }
    }
}

/// A call frame for function calls.
#[derive(Debug)]
struct CallFrame {
    /// The function being executed
    chunk: Chunk,
    /// Instruction pointer
    ip: usize,
    /// Base pointer (start of this frame's stack slots)
    bp: usize,
}

/// The Five virtual machine.
pub struct VM {
    /// The value stack
    stack: Vec<VMValue>,
    /// Call frames
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, VMValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new VM.
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
        };

        // Register native functions
        vm.define_native("print", 1, native_print);
        vm.define_native("println", 1, native_println);

        vm
    }

    /// Define a native function.
    pub fn define_native(
        &mut self,
        name: &str,
        arity: usize,
        func: fn(&[VMValue]) -> FiveResult<VMValue>,
    ) {
        self.globals.insert(
            name.to_string(),
            VMValue::NativeFunction {
                name: name.to_string(),
                arity,
                func,
            },
        );
    }

    /// Run a chunk of bytecode.
    pub fn run(&mut self, chunk: Chunk) -> FiveResult<VMValue> {
        self.frames.push(CallFrame {
            chunk,
            ip: 0,
            bp: 0,
        });

        self.execute()
    }

    /// Execute bytecode.
    fn execute(&mut self) -> FiveResult<VMValue> {
        loop {
            if self.frames.is_empty() {
                return Ok(self.stack.pop().unwrap_or(VMValue::Nil));
            }

            let frame = self.frames.last_mut().unwrap();

            if frame.ip >= frame.chunk.code.len() {
                // End of function
                let result = self.stack.pop().unwrap_or(VMValue::Nil);

                // Pop the frame
                let frame = self.frames.pop().unwrap();

                // Clean up stack
                self.stack.truncate(frame.bp);

                // Push result
                self.stack.push(result);

                continue;
            }

            let instruction = frame.chunk.code[frame.ip].clone();
            frame.ip += 1;

            match instruction.opcode {
                OpCode::Constant => {
                    let idx = instruction.operand.unwrap() as usize;
                    let value = frame.chunk.constants[idx].clone();
                    self.stack.push(value);
                }

                OpCode::Nil => {
                    self.stack.push(VMValue::Nil);
                }

                OpCode::True => {
                    self.stack.push(VMValue::Bool(true));
                }

                OpCode::False => {
                    self.stack.push(VMValue::Bool(false));
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::GetLocal => {
                    let slot = instruction.operand.unwrap() as usize;
                    let bp = frame.bp;
                    let value = self.stack[bp + slot].clone();
                    self.stack.push(value);
                }

                OpCode::SetLocal => {
                    let slot = instruction.operand.unwrap() as usize;
                    let bp = frame.bp;
                    let value = self.stack.last().unwrap().clone();
                    self.stack[bp + slot] = value;
                }

                OpCode::GetGlobal => {
                    let idx = instruction.operand.unwrap() as usize;
                    let name = match &frame.chunk.constants[idx] {
                        VMValue::String(s) => s.clone(),
                        _ => {
                            return Err(FiveError::runtime(
                                "Invalid global name",
                                Span::dummy(),
                            ))
                        }
                    };
                    let value = self.globals.get(&name).cloned().ok_or_else(|| {
                        FiveError::runtime(format!("Undefined variable: {}", name), Span::dummy())
                    })?;
                    self.stack.push(value);
                }

                OpCode::SetGlobal => {
                    let idx = instruction.operand.unwrap() as usize;
                    let name = match &frame.chunk.constants[idx] {
                        VMValue::String(s) => s.clone(),
                        _ => {
                            return Err(FiveError::runtime(
                                "Invalid global name",
                                Span::dummy(),
                            ))
                        }
                    };
                    let value = self.stack.last().unwrap().clone();
                    self.globals.insert(name, value);
                }

                OpCode::DefineGlobal => {
                    let idx = instruction.operand.unwrap() as usize;
                    let name = match &frame.chunk.constants[idx] {
                        VMValue::String(s) => s.clone(),
                        _ => {
                            return Err(FiveError::runtime(
                                "Invalid global name",
                                Span::dummy(),
                            ))
                        }
                    };
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                }

                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Int(a + b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Float(a + b),
                        (VMValue::String(a), VMValue::String(b)) => {
                            VMValue::String(format!("{}{}", a, b))
                        }
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot add {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Int(a - b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Float(a - b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot subtract {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Int(a * b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Float(a * b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot multiply {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => {
                            if *b == 0 {
                                return Err(FiveError::runtime("Division by zero", Span::dummy()));
                            }
                            VMValue::Int(a / b)
                        }
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Float(a / b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot divide {} by {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Modulo => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => {
                            if *b == 0 {
                                return Err(FiveError::runtime("Modulo by zero", Span::dummy()));
                            }
                            VMValue::Int(a % b)
                        }
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot modulo {} by {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Negate => {
                    let value = self.stack.pop().unwrap();
                    let result = match value {
                        VMValue::Int(n) => VMValue::Int(-n),
                        VMValue::Float(n) => VMValue::Float(-n),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot negate {}", value.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(VMValue::Bool(!value.is_truthy()));
                }

                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(VMValue::Bool(a == b));
                }

                OpCode::NotEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(VMValue::Bool(a != b));
                }

                OpCode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Bool(a < b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Bool(a < b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot compare {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::LessEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Bool(a <= b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Bool(a <= b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot compare {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Greater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Bool(a > b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Bool(a > b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot compare {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::GreaterEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (&a, &b) {
                        (VMValue::Int(a), VMValue::Int(b)) => VMValue::Bool(a >= b),
                        (VMValue::Float(a), VMValue::Float(b)) => VMValue::Bool(a >= b),
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot compare {} and {}", a.type_name(), b.type_name()),
                                Span::dummy(),
                            ))
                        }
                    };
                    self.stack.push(result);
                }

                OpCode::Jump => {
                    let offset = instruction.operand.unwrap() as usize;
                    self.frames.last_mut().unwrap().ip = offset;
                }

                OpCode::JumpIfFalse => {
                    let offset = instruction.operand.unwrap() as usize;
                    let condition = self.stack.last().unwrap();
                    if !condition.is_truthy() {
                        self.frames.last_mut().unwrap().ip = offset;
                    }
                }

                OpCode::JumpIfTrue => {
                    let offset = instruction.operand.unwrap() as usize;
                    let condition = self.stack.last().unwrap();
                    if condition.is_truthy() {
                        self.frames.last_mut().unwrap().ip = offset;
                    }
                }

                OpCode::Call => {
                    let arity = instruction.operand.unwrap() as usize;
                    let callee_idx = self.stack.len() - arity - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        VMValue::Function { chunk, .. } => {
                            let new_frame = CallFrame {
                                chunk,
                                ip: 0,
                                bp: callee_idx,
                            };
                            self.frames.push(new_frame);
                        }
                        VMValue::NativeFunction { func, arity: expected_arity, .. } => {
                            if arity != expected_arity {
                                return Err(FiveError::runtime(
                                    format!("Expected {} arguments, got {}", expected_arity, arity),
                                    Span::dummy(),
                                ));
                            }

                            let args: Vec<_> = self.stack.drain(callee_idx + 1..).collect();
                            self.stack.pop(); // Remove the function itself
                            let result = func(&args)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(FiveError::runtime(
                                format!("Cannot call {}", callee.type_name()),
                                Span::dummy(),
                            ))
                        }
                    }
                }

                OpCode::Return => {
                    let result = self.stack.pop().unwrap_or(VMValue::Nil);
                    let frame = self.frames.pop().unwrap();
                    self.stack.truncate(frame.bp);
                    self.stack.push(result);
                }

                OpCode::Array => {
                    let size = instruction.operand.unwrap() as usize;
                    let start = self.stack.len() - size;
                    let elements: Vec<_> = self.stack.drain(start..).collect();
                    self.stack.push(VMValue::Array(elements));
                }

                OpCode::Index => {
                    let index = self.stack.pop().unwrap();
                    let array = self.stack.pop().unwrap();

                    match (&array, &index) {
                        (VMValue::Array(arr), VMValue::Int(i)) => {
                            let idx = *i as usize;
                            if idx >= arr.len() {
                                return Err(FiveError::runtime(
                                    format!("Index {} out of bounds", idx),
                                    Span::dummy(),
                                ));
                            }
                            self.stack.push(arr[idx].clone());
                        }
                        _ => {
                            return Err(FiveError::runtime(
                                format!(
                                    "Cannot index {} with {}",
                                    array.type_name(),
                                    index.type_name()
                                ),
                                Span::dummy(),
                            ))
                        }
                    }
                }
            }
        }
    }
}

fn native_print(args: &[VMValue]) -> FiveResult<VMValue> {
    for arg in args {
        print!("{}", arg);
    }
    Ok(VMValue::Nil)
}

fn native_println(args: &[VMValue]) -> FiveResult<VMValue> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(VMValue::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();

        // 1 + 2
        chunk.add_constant(VMValue::Int(1));
        chunk.add_constant(VMValue::Int(2));
        chunk.write(Instruction::new(OpCode::Constant, Some(0)), 1);
        chunk.write(Instruction::new(OpCode::Constant, Some(1)), 1);
        chunk.write(Instruction::new(OpCode::Add, None), 1);

        let result = vm.run(chunk).unwrap();
        assert_eq!(result, VMValue::Int(3));
    }
}
