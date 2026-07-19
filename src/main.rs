//! Five - A next-generation, AI-native programming language.

use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::{Parser as ClapParser, Subcommand};
use five_compiler::Compiler;
use five_core::FiveError;
use five_effects::EffectChecker;
use five_interpreter::Interpreter;
use five_parser::Parser;
use five_types::TypeChecker;
use five_vm::VM;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "five")]
#[command(about = "Five - A next-generation, AI-native programming language", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Five program with the interpreter
    Run {
        /// The file to run
        file: PathBuf,
    },
    /// Compile a Five program to bytecode and run it
    Build {
        /// The file to compile
        file: PathBuf,
    },
    /// Start the interactive REPL
    Repl,
    /// Type check a Five program
    Check {
        /// The file to check
        file: PathBuf,
    },
    /// Format a Five program (not yet implemented)
    Fmt {
        /// The file to format
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run_file(&file),
        Commands::Build { file } => build_file(&file),
        Commands::Repl => run_repl(),
        Commands::Check { file } => check_file(&file),
        Commands::Fmt { file } => fmt_file(&file),
    }
}

fn run_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut interpreter = Interpreter::new();

    match interpreter.run(&source) {
        Ok(value) => {
            // Only print if not nil
            if !matches!(value, five_interpreter::Value::Nil) {
                println!("{}", value);
            }
            Ok(())
        }
        Err(err) => {
            print_error(&source, path.to_string_lossy().as_ref(), &err);
            std::process::exit(1);
        }
    }
}

fn build_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse
    let program = match Parser::parse(&source) {
        Ok(p) => p,
        Err(err) => {
            print_error(&source, path.to_string_lossy().as_ref(), &err);
            std::process::exit(1);
        }
    };

    // Compile
    let mut compiler = Compiler::new();
    let chunk = match compiler.compile_program(&program) {
        Ok(c) => c,
        Err(err) => {
            print_error(&source, path.to_string_lossy().as_ref(), &err);
            std::process::exit(1);
        }
    };

    // Run in VM
    let mut vm = VM::new();
    match vm.run(chunk) {
        Ok(value) => {
            if !matches!(value, five_vm::VMValue::Nil) {
                println!("{}", value);
            }
            Ok(())
        }
        Err(err) => {
            print_error(&source, path.to_string_lossy().as_ref(), &err);
            std::process::exit(1);
        }
    }
}

fn run_repl() -> Result<()> {
    println!("Five REPL v0.1.0");
    println!("Type :help for help, :quit to exit");
    println!();

    let mut rl = DefaultEditor::new()?;
    let mut interpreter = Interpreter::new();
    let mut multiline_buffer = String::new();

    loop {
        let prompt = if multiline_buffer.is_empty() {
            "five> "
        } else {
            "  ... "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                // Handle REPL commands
                if multiline_buffer.is_empty() {
                    let trimmed = line.trim();
                    if trimmed == ":quit" || trimmed == ":q" {
                        println!("Goodbye!");
                        break;
                    }
                    if trimmed == ":help" || trimmed == ":h" {
                        print_repl_help();
                        continue;
                    }
                    if trimmed == ":clear" || trimmed == ":c" {
                        interpreter = Interpreter::new();
                        println!("Environment cleared.");
                        continue;
                    }
                }

                multiline_buffer.push_str(&line);
                multiline_buffer.push('\n');

                // Try to parse - if incomplete, continue
                match Parser::parse(&multiline_buffer) {
                    Ok(program) => {
                        let _ = rl.add_history_entry(multiline_buffer.trim());

                        match interpreter.eval_program(&program) {
                            Ok(value) => {
                                if !matches!(value, five_interpreter::Value::Nil) {
                                    println!("=> {}", value);
                                }
                            }
                            Err(err) => {
                                print_error(&multiline_buffer, "<repl>", &err);
                            }
                        }

                        multiline_buffer.clear();
                    }
                    Err(err) => {
                        // Check if it looks like an incomplete expression
                        let msg = err.message();
                        if msg.contains("end of file") || msg.contains("Expected '}'") {
                            // Probably incomplete, wait for more input
                            continue;
                        }

                        // Otherwise, report the error
                        let _ = rl.add_history_entry(multiline_buffer.trim());
                        print_error(&multiline_buffer, "<repl>", &err);
                        multiline_buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                multiline_buffer.clear();
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn check_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse
    let program = match Parser::parse(&source) {
        Ok(p) => p,
        Err(err) => {
            print_error(&source, path.to_string_lossy().as_ref(), &err);
            std::process::exit(1);
        }
    };

    // Type check
    let mut type_checker = TypeChecker::new();
    if let Err(err) = type_checker.check_program(&program) {
        print_error(&source, path.to_string_lossy().as_ref(), &err);
        std::process::exit(1);
    }

    // Effect check
    let mut effect_checker = EffectChecker::new();
    if let Err(err) = effect_checker.check_program(&program) {
        print_error(&source, path.to_string_lossy().as_ref(), &err);
        std::process::exit(1);
    }

    println!("OK: No errors found in {}", path.display());
    Ok(())
}

fn fmt_file(path: &PathBuf) -> Result<()> {
    println!("Formatting not yet implemented for {}", path.display());
    Ok(())
}

fn print_repl_help() {
    println!("Five REPL Commands:");
    println!("  :help, :h     Show this help message");
    println!("  :quit, :q     Exit the REPL");
    println!("  :clear, :c    Clear the environment");
    println!();
    println!("Examples:");
    println!("  let x = 42");
    println!("  fn double(n) {{ n * 2 }}");
    println!("  [1, 2, 3].map((x) => x * 2)");
    println!();
}

fn print_error(source: &str, filename: &str, err: &FiveError) {
    if let Some(span) = err.span() {
        Report::build(ReportKind::Error, filename, span.start)
            .with_message(err.message())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(err.message())
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((filename, Source::from(source)))
            .unwrap();
    } else {
        eprintln!("Error: {}", err);
    }
}
