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

    // Math functions
    env.define(
        "abs".to_string(),
        Value::BuiltinFunction {
            name: "abs".to_string(),
            func: builtin_abs,
        },
    );

    env.define(
        "min".to_string(),
        Value::BuiltinFunction {
            name: "min".to_string(),
            func: builtin_min,
        },
    );

    env.define(
        "max".to_string(),
        Value::BuiltinFunction {
            name: "max".to_string(),
            func: builtin_max,
        },
    );

    env.define(
        "sqrt".to_string(),
        Value::BuiltinFunction {
            name: "sqrt".to_string(),
            func: builtin_sqrt,
        },
    );

    env.define(
        "pow".to_string(),
        Value::BuiltinFunction {
            name: "pow".to_string(),
            func: builtin_pow,
        },
    );

    env.define(
        "floor".to_string(),
        Value::BuiltinFunction {
            name: "floor".to_string(),
            func: builtin_floor,
        },
    );

    env.define(
        "ceil".to_string(),
        Value::BuiltinFunction {
            name: "ceil".to_string(),
            func: builtin_ceil,
        },
    );

    env.define(
        "round".to_string(),
        Value::BuiltinFunction {
            name: "round".to_string(),
            func: builtin_round,
        },
    );

    // Random functions
    env.define(
        "random".to_string(),
        Value::BuiltinFunction {
            name: "random".to_string(),
            func: builtin_random,
        },
    );

    env.define(
        "random_int".to_string(),
        Value::BuiltinFunction {
            name: "random_int".to_string(),
            func: builtin_random_int,
        },
    );

    // File I/O
    env.define(
        "read_file".to_string(),
        Value::BuiltinFunction {
            name: "read_file".to_string(),
            func: builtin_read_file,
        },
    );

    env.define(
        "write_file".to_string(),
        Value::BuiltinFunction {
            name: "write_file".to_string(),
            func: builtin_write_file,
        },
    );

    env.define(
        "file_exists".to_string(),
        Value::BuiltinFunction {
            name: "file_exists".to_string(),
            func: builtin_file_exists,
        },
    );

    // System functions
    env.define(
        "env".to_string(),
        Value::BuiltinFunction {
            name: "env".to_string(),
            func: builtin_env,
        },
    );

    env.define(
        "sleep".to_string(),
        Value::BuiltinFunction {
            name: "sleep".to_string(),
            func: builtin_sleep,
        },
    );

    env.define(
        "exit".to_string(),
        Value::BuiltinFunction {
            name: "exit".to_string(),
            func: builtin_exit,
        },
    );

    env.define(
        "time".to_string(),
        Value::BuiltinFunction {
            name: "time".to_string(),
            func: builtin_time,
        },
    );

    // Error handling
    env.define(
        "panic".to_string(),
        Value::BuiltinFunction {
            name: "panic".to_string(),
            func: builtin_panic,
        },
    );

    // Map/Dictionary functions
    env.define(
        "map".to_string(),
        Value::BuiltinFunction {
            name: "map".to_string(),
            func: builtin_map_new,
        },
    );

    env.define(
        "keys".to_string(),
        Value::BuiltinFunction {
            name: "keys".to_string(),
            func: builtin_keys,
        },
    );

    env.define(
        "values".to_string(),
        Value::BuiltinFunction {
            name: "values".to_string(),
            func: builtin_values,
        },
    );

    env.define(
        "has_key".to_string(),
        Value::BuiltinFunction {
            name: "has_key".to_string(),
            func: builtin_has_key,
        },
    );

    env.define(
        "get".to_string(),
        Value::BuiltinFunction {
            name: "get".to_string(),
            func: builtin_get,
        },
    );

    env.define(
        "set".to_string(),
        Value::BuiltinFunction {
            name: "set".to_string(),
            func: builtin_set,
        },
    );

    // Functional helpers
    env.define(
        "zip".to_string(),
        Value::BuiltinFunction {
            name: "zip".to_string(),
            func: builtin_zip,
        },
    );

    env.define(
        "enumerate".to_string(),
        Value::BuiltinFunction {
            name: "enumerate".to_string(),
            func: builtin_enumerate,
        },
    );

    env.define(
        "sort".to_string(),
        Value::BuiltinFunction {
            name: "sort".to_string(),
            func: builtin_sort,
        },
    );

    env.define(
        "flatten".to_string(),
        Value::BuiltinFunction {
            name: "flatten".to_string(),
            func: builtin_flatten,
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

// Math functions

fn builtin_abs(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("abs() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err(FiveError::runtime(
            format!("abs() requires a number, got {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_min(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() < 2 {
        return Err(FiveError::runtime("min() requires at least 2 arguments", span));
    }

    let mut result = args[0].clone();
    for arg in &args[1..] {
        match (&result, arg) {
            (Value::Int(a), Value::Int(b)) => {
                if b < a {
                    result = Value::Int(*b);
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b < a {
                    result = Value::Float(*b);
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let a_f = *a as f64;
                if *b < a_f {
                    result = Value::Float(*b);
                } else {
                    result = Value::Float(a_f);
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let b_f = *b as f64;
                if b_f < *a {
                    result = Value::Float(b_f);
                }
            }
            _ => {
                return Err(FiveError::runtime(
                    "min() requires numeric arguments",
                    span,
                ))
            }
        }
    }
    Ok(result)
}

fn builtin_max(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() < 2 {
        return Err(FiveError::runtime("max() requires at least 2 arguments", span));
    }

    let mut result = args[0].clone();
    for arg in &args[1..] {
        match (&result, arg) {
            (Value::Int(a), Value::Int(b)) => {
                if b > a {
                    result = Value::Int(*b);
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b > a {
                    result = Value::Float(*b);
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let a_f = *a as f64;
                if *b > a_f {
                    result = Value::Float(*b);
                } else {
                    result = Value::Float(a_f);
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let b_f = *b as f64;
                if b_f > *a {
                    result = Value::Float(b_f);
                }
            }
            _ => {
                return Err(FiveError::runtime(
                    "max() requires numeric arguments",
                    span,
                ))
            }
        }
    }
    Ok(result)
}

fn builtin_sqrt(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("sqrt() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(FiveError::runtime(
            format!("sqrt() requires a number, got {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_pow(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 2 {
        return Err(FiveError::runtime("pow() takes exactly 2 arguments", span));
    }

    match (&args[0], &args[1]) {
        (Value::Int(base), Value::Int(exp)) => {
            if *exp >= 0 {
                Ok(Value::Int(base.pow(*exp as u32)))
            } else {
                Ok(Value::Float((*base as f64).powi(*exp as i32)))
            }
        }
        (Value::Float(base), Value::Int(exp)) => Ok(Value::Float(base.powi(*exp as i32))),
        (Value::Int(base), Value::Float(exp)) => Ok(Value::Float((*base as f64).powf(*exp))),
        (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
        _ => Err(FiveError::runtime(
            "pow() requires numeric arguments",
            span,
        )),
    }
}

fn builtin_floor(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("floor() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(n) => Ok(Value::Int(n.floor() as i64)),
        _ => Err(FiveError::runtime(
            format!("floor() requires a number, got {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_ceil(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("ceil() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(n) => Ok(Value::Int(n.ceil() as i64)),
        _ => Err(FiveError::runtime(
            format!("ceil() requires a number, got {}", args[0].type_name()),
            span,
        )),
    }
}

fn builtin_round(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("round() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(n) => Ok(Value::Int(n.round() as i64)),
        _ => Err(FiveError::runtime(
            format!("round() requires a number, got {}", args[0].type_name()),
            span,
        )),
    }
}

// Random functions

fn builtin_random(_args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Simple PRNG using time-based seed
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random_val = ((seed as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) as f64
        / u64::MAX as f64;
    Ok(Value::Float(random_val))
}

fn builtin_random_int(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (min, max) = match args.len() {
        0 => (0i64, i64::MAX),
        1 => {
            if let Value::Int(max) = &args[0] {
                (0, *max)
            } else {
                return Err(FiveError::runtime("random_int() requires integer arguments", span));
            }
        }
        2 => {
            if let (Value::Int(min), Value::Int(max)) = (&args[0], &args[1]) {
                (*min, *max)
            } else {
                return Err(FiveError::runtime("random_int() requires integer arguments", span));
            }
        }
        _ => return Err(FiveError::runtime("random_int() takes 0, 1, or 2 arguments", span)),
    };

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random_val = (seed as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let range = (max - min) as u64;
    let result = if range > 0 {
        min + (random_val % range) as i64
    } else {
        min
    };
    Ok(Value::Int(result))
}

// File I/O

fn builtin_read_file(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("read_file() takes exactly 1 argument", span));
    }

    if let Value::String(path) = &args[0] {
        match std::fs::read_to_string(path) {
            Ok(contents) => Ok(Value::String(contents)),
            Err(e) => Err(FiveError::runtime(format!("Failed to read file: {}", e), span)),
        }
    } else {
        Err(FiveError::runtime("read_file() requires a string path", span))
    }
}

fn builtin_write_file(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 2 {
        return Err(FiveError::runtime("write_file() takes exactly 2 arguments", span));
    }

    if let (Value::String(path), Value::String(contents)) = (&args[0], &args[1]) {
        match std::fs::write(path, contents) {
            Ok(()) => Ok(Value::Bool(true)),
            Err(e) => Err(FiveError::runtime(format!("Failed to write file: {}", e), span)),
        }
    } else {
        Err(FiveError::runtime("write_file() requires (string, string) arguments", span))
    }
}

fn builtin_file_exists(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("file_exists() takes exactly 1 argument", span));
    }

    if let Value::String(path) = &args[0] {
        Ok(Value::Bool(std::path::Path::new(path).exists()))
    } else {
        Err(FiveError::runtime("file_exists() requires a string path", span))
    }
}

// System functions

fn builtin_env(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("env() takes exactly 1 argument", span));
    }

    if let Value::String(name) = &args[0] {
        match std::env::var(name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(Value::Nil),
        }
    } else {
        Err(FiveError::runtime("env() requires a string argument", span))
    }
}

fn builtin_sleep(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("sleep() takes exactly 1 argument", span));
    }

    let millis = match &args[0] {
        Value::Int(n) => *n as u64,
        Value::Float(n) => (*n * 1000.0) as u64,
        _ => return Err(FiveError::runtime("sleep() requires a number (seconds)", span)),
    };

    std::thread::sleep(std::time::Duration::from_millis(millis));
    Ok(Value::Nil)
}

fn builtin_exit(args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    let code = if let Some(Value::Int(n)) = args.first() {
        *n as i32
    } else {
        0
    };
    std::process::exit(code);
}

fn builtin_time(_args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    Ok(Value::Float(now.as_secs_f64()))
}

fn builtin_panic(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    let message = if let Some(arg) = args.first() {
        format!("{}", arg)
    } else {
        "panic!".to_string()
    };
    Err(FiveError::runtime(format!("Panic: {}", message), span))
}

// Map/Dictionary functions

fn builtin_map_new(_args: Vec<Value>, _span: Span) -> FiveResult<Value> {
    Ok(Value::Map(Vec::new()))
}

fn builtin_keys(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("keys() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Map(entries) => {
            Ok(Value::Array(entries.iter().map(|(k, _)| k.clone()).collect()))
        }
        Value::Struct { fields, .. } => {
            Ok(Value::Array(fields.keys().map(|k| Value::String(k.clone())).collect()))
        }
        _ => Err(FiveError::runtime("keys() requires a map or struct", span)),
    }
}

fn builtin_values(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("values() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Map(entries) => {
            Ok(Value::Array(entries.iter().map(|(_, v)| v.clone()).collect()))
        }
        Value::Struct { fields, .. } => {
            Ok(Value::Array(fields.values().cloned().collect()))
        }
        _ => Err(FiveError::runtime("values() requires a map or struct", span)),
    }
}

fn builtin_has_key(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 2 {
        return Err(FiveError::runtime("has_key() takes exactly 2 arguments", span));
    }

    match &args[0] {
        Value::Map(entries) => {
            let key = &args[1];
            Ok(Value::Bool(entries.iter().any(|(k, _)| k == key)))
        }
        Value::Struct { fields, .. } => {
            if let Value::String(key) = &args[1] {
                Ok(Value::Bool(fields.contains_key(key)))
            } else {
                Err(FiveError::runtime("struct key must be a string", span))
            }
        }
        _ => Err(FiveError::runtime("has_key() requires a map or struct as first argument", span)),
    }
}

fn builtin_get(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(FiveError::runtime("get() takes 2 or 3 arguments", span));
    }

    let default = if args.len() == 3 {
        args[2].clone()
    } else {
        Value::Nil
    };

    match &args[0] {
        Value::Map(entries) => {
            let key = &args[1];
            Ok(entries.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or(default))
        }
        Value::Struct { fields, .. } => {
            if let Value::String(key) = &args[1] {
                Ok(fields.get(key).cloned().unwrap_or(default))
            } else {
                Err(FiveError::runtime("struct key must be a string", span))
            }
        }
        Value::Array(arr) => {
            if let Value::Int(idx) = &args[1] {
                Ok(arr.get(*idx as usize).cloned().unwrap_or(default))
            } else {
                Err(FiveError::runtime("array index must be an integer", span))
            }
        }
        _ => Err(FiveError::runtime("get() requires a map, struct, or array", span)),
    }
}

fn builtin_set(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 3 {
        return Err(FiveError::runtime("set() takes exactly 3 arguments", span));
    }

    match &args[0] {
        Value::Map(entries) => {
            let key = args[1].clone();
            let value = args[2].clone();
            let mut new_entries: Vec<(Value, Value)> = entries.iter()
                .filter(|(k, _)| k != &key)
                .cloned()
                .collect();
            new_entries.push((key, value));
            Ok(Value::Map(new_entries))
        }
        _ => Err(FiveError::runtime("set() requires a map as first argument", span)),
    }
}

// Functional helpers

fn builtin_zip(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 2 {
        return Err(FiveError::runtime("zip() takes exactly 2 arguments", span));
    }

    match (&args[0], &args[1]) {
        (Value::Array(a), Value::Array(b)) => {
            let result: Vec<Value> = a.iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err(FiveError::runtime("zip() requires two arrays", span)),
    }
}

fn builtin_enumerate(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("enumerate() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Array(arr) => {
            let result: Vec<Value> = arr.iter()
                .enumerate()
                .map(|(i, v)| Value::Tuple(vec![Value::Int(i as i64), v.clone()]))
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err(FiveError::runtime("enumerate() requires an array", span)),
    }
}

fn builtin_sort(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("sort() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut sorted = arr.clone();
            sorted.sort_by(|a, b| {
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::String(x), Value::String(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            Ok(Value::Array(sorted))
        }
        _ => Err(FiveError::runtime("sort() requires an array", span)),
    }
}

fn builtin_flatten(args: Vec<Value>, span: Span) -> FiveResult<Value> {
    if args.len() != 1 {
        return Err(FiveError::runtime("flatten() takes exactly 1 argument", span));
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                match item {
                    Value::Array(inner) => result.extend(inner.clone()),
                    other => result.push(other.clone()),
                }
            }
            Ok(Value::Array(result))
        }
        _ => Err(FiveError::runtime("flatten() requires an array", span)),
    }
}
