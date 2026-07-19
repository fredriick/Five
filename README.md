# Five Programming Language

A next-generation, AI-native, general-purpose programming language with a hybrid execution model.

## Features

- **Gradual Typing** - Start dynamic, add types as needed
- **Effect Tracking** - Know what code does (IO, State, Async)
- **AI-Native** - AST as first-class data, semantic annotations
- **Hybrid Execution** - Interpret for development, compile for production
- **Ownership System** - Rust-like memory safety without GC
- **Pipe Operator** - Clean function composition with `|>`

## Quick Start

```bash
# Run a Five program with the interpreter
cargo run -- run examples/hello.five

# Compile and run with the VM
cargo run -- build examples/hello.five

# Start the interactive REPL
cargo run -- repl

# Type check a program
cargo run -- check examples/hello.five
```

## Example

```five
// Variables with gradual typing
let name = "Five"
let age: Int = 1

// Functions with effect tracking
fn greet(name: String) -> String {
    "Hello, " + name
}

fn readFile(path: String) -> String with IO {
    // IO operations are tracked
    File.read(path)
}

// Algebraic data types
data Option<T> {
    Some(T),
    None
}

// Pattern matching
fn unwrap<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(value) => value,
        None => default
    }
}

// Pipe operator for composition
let result = data
    |> parse
    |> validate
    |> transform

// Lambdas
let double = (x) => x * 2
let nums = [1, 2, 3].map((x) => x * 2)
```

## Project Structure

```
five/
├── crates/
│   ├── five_core/       # Shared utilities & errors
│   ├── five_ast/        # AST definitions
│   ├── five_lexer/      # Tokenization
│   ├── five_parser/     # Parsing
│   ├── five_types/      # Type inference
│   ├── five_effects/    # Effect tracking
│   ├── five_interpreter/# Tree-walking interpreter
│   ├── five_compiler/   # Bytecode compilation
│   └── five_vm/         # Virtual machine
├── src/main.rs          # CLI entry point
├── examples/            # Example programs
└── stdlib/              # Standard library (Five code)
```

## Running Tests

```bash
cargo test --workspace
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `five run <file>` | Run with interpreter |
| `five build <file>` | Compile to bytecode and run |
| `five repl` | Interactive REPL |
| `five check <file>` | Type check only |
| `five fmt <file>` | Format code (coming soon) |

## Language Features

### Ownership System

```five
let text = String.from("hello")
let borrowed = &text           // immutable borrow
let len = borrowed.len()       // use the borrow
print(text)                    // original still valid

let mut buffer = String.new()
buffer.push("hi")              // mutate owned value
let edit = &mut buffer         // mutable borrow
edit.push("!")                 // mutate through borrow
```

### Effect System

```five
// Pure function
fn add(a: Int, b: Int) -> Int {
    a + b
}

// Function with IO effect
fn greet(name: String) with IO {
    println("Hello, " + name)
}

// Async function
async fn fetchUser(id: Int) -> User with IO, Async {
    let response = await Http.get("/users/" + id)
    Json.parse(response.body)
}
```

## License

MIT
