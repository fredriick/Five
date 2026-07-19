//! Built-in functions for the Five interpreter.

use crate::{Environment, Value};
use five_core::{FiveError, FiveResult, Span};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all built-in functions in the environment.
pub fn register_builtins(env: &Rc<RefCell<Environment>>) {
    let mut env = env.borrow_mut();

    // I/O functions
    env.define(
        "print".to_string(),
        Value::BuiltinFunction {
            name: "print".to_string(),
            func: builtin_print,
        },
    );

    env.define(
        "println".to_string(),
        Value::BuiltinFunction {
            name: "println".to_string(),
            func: builtin_println,
        },
    );

    env.define(
        "input".to_string(),
        Value::BuiltinFunction {
            name: "input".to_string(),
            func: builtin_input,
        },
    );

    // Type conversion
    env.define(
        "int".to_string(),
        Value::BuiltinFunction {
            name: "int".to_string(),
            func: builtin_int,
        },
    );

    env.define(
        "float".to_string(),
        Value::BuiltinFunction {
            name: "float".to_string(),
            func: builtin_float,
        },
    );

    env.define(
        "string".to_string(),
        Value::BuiltinFunction {
            name: "string".to_string(),
            func: builtin_string,
        },
    );

    // Collection functions
    env.define(
        "len".to_string(),
        Value::BuiltinFunction {
            name: "len".to_string(),
            func: builtin_len,
        },
    );

    env.define(
        "push".to_string(),
        Value::BuiltinFunction {
            name: "push".to_string(),
            func: builtin_push,
        },
    );

    env.define(
        "pop".to_string(),
        Value::BuiltinFunction {
            name: "pop".to_string(),
            func: builtin_pop,
        },
    );

    env.define(
        "range".to_string(),
        Value::BuiltinFunction {
            name: "range".to_string(),
            func: builtin_range,
        },
    );

    // Type checking
    env.define(
        "type_of".to_string(),
        Value::BuiltinFunction {
            name: "type_of".to_string(),
            func: builtin_type_of,
        },
    );

    // Debug
    env.define(
        "debug".to_string(),
        Value::BuiltinFunction {
            name: "debug".to_string(),
            func: builtin_debug,
        },
    );

    // Assertions
    env.define(
        "assert".to_string(),
        Value::BuiltinFunction {
            name: "assert".to_string(),
            func: builtin_assert,
        },
    );

    env.define(
        "assert_eq".to_string(),
        Value::BuiltinFunction {
            name: "assert_eq".to_string(),
            func: builtin_assert_eq,
        },
    );
}

fn builtin_print(args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    Ok(Value::Nil)
}

fn builtin_println(args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

fn builtin_input(args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    // Print prompt if provided
    if let Some(prompt) = args.first() {
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(Value::String(input.trim_end().to_string()))
}

fn builtin_int(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("int() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(n) => Ok(Value::Int(*n as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| FiveError::runtime(format!("Cannot convert '{}' to int", s), span)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Err(FiveError::runtime(
            format!("Cannot convert {} to int", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_float(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("float() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(n) => Ok(Value::Float(*n)),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| FiveError::runtime(format!("Cannot convert '{}' to float", s), span)),
        _ => Err(FiveError::runtime(
            format!("Cannot convert {} to float", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_string(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime(
            "string() takes exactly 1 argument",
            span,
        ));
    }

    Ok(Value::String(format!("{}", args[0])))
}

fn builtin_len(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("len() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        _ => Err(FiveError::runtime(
            format!("Cannot get length of {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_push(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 2 {
        return Err(FiveError::runtime("push() takes exactly 2 arguments", span));
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[1].clone());
            Ok(Value::Array(new_arr))
        }
        _ => Err(FiveError::runtime(
            format!("Cannot push to {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_pop(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("pop() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Nil)
            } else {
                let mut new_arr = arr.clone();
                let popped = new_arr.pop().unwrap();
                // Return tuple of (popped_value, new_array)
                Ok(Value::Tuple(vec![popped, Value::Array(new_arr)]))
            }
        }
        _ => Err(FiveError::runtime(
            format!("Cannot pop from {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_range(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    match args.len() {
        1 => {
            // range(end)
            if let Value::Int(end) = &args[0] {
                Ok(Value::Range {
                    start: 0,
                    end: *end,
                    inclusive: false,
                })
            } else {
                Err(FiveError::runtime("range() arguments must be integers", span))
            }
        }
        2 => {
            // range(start, end)
            if let (Value::Int(start), Value::Int(end)) = (&args[0], &args[1]) {
                Ok(Value::Range {
                    start: *start,
                    end: *end,
                    inclusive: false,
                })
            } else {
                Err(FiveError::runtime("range() arguments must be integers", span))
            }
        }
        3 => {
            // range(start, end, inclusive)
            if let (Value::Int(start), Value::Int(end), Value::Bool(inclusive)) =
                (&args[0], &args[1], &args[2])
            {
                Ok(Value::Range {
                    start: *start,
                    end: *end,
                    inclusive: *inclusive,
                })
            } else {
                Err(FiveError::runtime(
                    "range() arguments must be (int, int, bool)",
                    span,
                ))
            }
        }
        _ => Err(FiveError::runtime(
            "range() takes 1, 2, or 3 arguments",
            span,
        )),
    }
}

fn builtin_type_of(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime(
            "type_of() takes exactly 1 argument",
            span,
        ));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}

fn builtin_debug(args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{:?}", arg);
    }
    println!();
    Ok(Value::Nil)
}

fn builtin_assert(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.is_empty() {
        return Err(FiveError::runtime(
            "assert() requires at least 1 argument",
            span,
        ));
    }

    if !args[0].is_truthy() {
        let message = if args.len() > 1 {
            format!("Assertion failed: {}", args[1])
        } else {
            "Assertion failed".to_string()
        };
        return Err(FiveError::runtime(message, span));
    }

    Ok(Value::Nil)
}

fn builtin_assert_eq(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() < 2 {
        return Err(FiveError::runtime(
            "assert_eq() requires at least 2 arguments",
            span,
        ));
    }

    if args[0] != args[1] {
        let message = if args.len() > 2 {
            format!(
                "Assertion failed: {} != {} - {}",
                args[0], args[1], args[2]
            )
        } else {
            format!("Assertion failed: {:?} != {:?}", args[0], args[1])
        };
        return Err(FiveError::runtime(message, span));
    }

    Ok(Value::Nil)
}
