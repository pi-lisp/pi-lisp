use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use pilisp::cubical;
use pilisp::eval::{eval, with_import_base};
use pilisp::gc::{GcHandle, Heap};
use pilisp::reader::parse_all;
use pilisp::helper::shared_read_line;
use pilisp::vm;

/// Parses and evaluates all expressions in `src`, printing each result.
///
/// `global_env` is passed as the GC root on every `eval` call so that the
/// mark phase never frees the global frame, even when a cycle fires mid-run.
fn run(src: &str, global_env: GcHandle, heap: &mut Heap) {
    match parse_all(src) {
        Ok(exprs) => {
            for e in exprs {
                match eval(&e, global_env, heap) {
                    Ok(result) => println!("=> {:?}", result),
                    Err(err) => println!("Evaluation Error: {}", err),
                }
            }
        }
        Err(err) => println!("Parse error: {}", err),
    }
}

fn repl(global_env: GcHandle, heap: &mut Heap) {
    const R: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const MAGENTA: &str = "\x1b[35m";
    const CYAN: &str = "\x1b[36m";

    println!("    /\\___/\\");
    println!("   (  UwU  )");
    println!("    \\ ({BOLD}{MAGENTA}\u{3c0}{R}) /");
    println!("    (_____)");
    println!("   {BOLD}{CYAN}pi-lisp REPL{R}");
    println!("   {DIM}Ctrl+D to exit{R}");
    let mut input = String::new();

    loop {
        let prompt = if input.is_empty() { "\u{3c0}> " } else { "...  " };
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        match shared_read_line() {
            Ok(None) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(Some(line)) => {
                input.push_str(&line);
                input.push('\n');

                if is_balanced(&input) {
                    let src = input.trim().to_string();
                    if !src.is_empty() {
                        run(&src, global_env, heap);
                    }
                    input.clear();
                }
            }
            Err(err) => {
                eprintln!("Input error: {}", err);
                break;
            }
        }
    }
}

/// Returns true when open parens are fully closed and input is non-empty.
fn is_balanced(src: &str) -> bool {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in src.chars() {
        if escape {
            escape = false;
            continue;
        }
        match ch {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '(' | '[' if !in_string => depth += 1,
            ')' | ']' if !in_string => depth -= 1,
            _ => {}
        }
    }

    depth <= 0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut heap = Heap::new();
    let global_env = pilisp::builtins::global_env(&mut heap);

    if args.len() < 2 {
        repl(global_env, &mut heap);
    } else if args[1] == "--help" || args[1] == "-h" {
        println!("pi-lisp \u{2014} a Lisp interpreter with cubical type theory and AOT compilation");
        println!();
        println!("USAGE:");
        println!("  pilisp                         Start the REPL");
        println!("  pilisp <file.pi>               Evaluate a Lisp source file");
        println!("  pilisp <file.pic>              Run a cubical type theory file");
        println!("  pilisp --cubical <file.pic>    Explicit cubical mode");
        println!("  pilisp --aot <file.pi>         AOT-compile to <file>.aot");
        println!("  pilisp --aot <file.pi> -o <f>  AOT-compile to <f>");
        println!("  pilisp --cubical-transpile <file.pic> [-o <dir>]     Transpile cubical to Python");
        println!("  pilisp --cubical-transpile-rust <file.pic> [-o <dir>] Transpile cubical to Rust");
        println!("  pilisp --help                  Show this help");
        println!();
        println!("When a .pi file is run, pilisp auto-loads <file>.aot if it exists");
        println!("for faster startup via cached bytecode.");
    } else {
        let mut i = 1;
        while i < args.len() {
            if args[i] == "--aot-compile" || args[i] == "--aot" {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: {} requires a filename argument", args[i - 1]);
                    process::exit(1);
                }
                let input_path = args[i].clone();
                i += 1;
                let output_path = if i < args.len() && args[i] == "-o" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("Error: -o requires a filename argument");
                        process::exit(1);
                    }
                    args[i].clone()
                } else {
                    let p = Path::new(&input_path);
                    let stem = p.file_stem().unwrap_or_default().to_str().unwrap_or("out");
                    format!("{}.aot", stem)
                };
                match vm::aot_compile_file(&input_path, &output_path, global_env, &mut heap) {
                    Ok(()) => println!("AOT compiled: {} -> {}", input_path, output_path),
                    Err(e) => eprintln!("AOT compile error: {}", e),
                }
                return;
            } else if args[i] == "--cubical" {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --cubical requires a filename argument");
                    process::exit(1);
                }
                match cubical::run(&args[i]) {
                    Ok(output) => println!("{}", output),
                    Err(err) => eprintln!("Cubical error: {}", err),
                }
            } else if args[i] == "--cubical-transpile" {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --cubical-transpile requires a filename argument");
                    process::exit(1);
                }
                let input_path = Path::new(&args[i]);
                let mut out_dir = input_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                i += 1;
                if i < args.len() && args[i] == "-o" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("Error: -o requires an output directory argument");
                        process::exit(1);
                    }
                    out_dir = PathBuf::from(&args[i]);
                }
                match cubical::transpile(input_path) {
                    Ok(output) => {
                        if let Err(err) = cubical::write_output(&output, &out_dir) {
                            eprintln!("Transpile write error: {}", err);
                            process::exit(1);
                        }
                        for module in &output.modules {
                            println!(
                                "wrote {}",
                                out_dir
                                    .join(module.path.file_name().unwrap_or_default())
                                    .display()
                            );
                        }
                        if output
                            .modules
                            .iter()
                            .any(|m| m.path.file_stem().and_then(|s| s.to_str()) == Some("Main"))
                        {
                            println!(
                                "run: cd {} && python main.py",
                                out_dir.display()
                            );
                        }
                    }
                    Err(err) => {
                        eprintln!("Transpile error: {}", err);
                        process::exit(1);
                    }
                }
            } else if args[i] == "--cubical-transpile-rust" {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --cubical-transpile-rust requires a filename argument");
                    process::exit(1);
                }
                let input_path = Path::new(&args[i]);
                let mut out_dir = input_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                i += 1;
                if i < args.len() && args[i] == "-o" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("Error: -o requires an output directory argument");
                        process::exit(1);
                    }
                    out_dir = PathBuf::from(&args[i]);
                }
                match cubical::transpile_rust(input_path) {
                    Ok(output) => {
                        if let Err(err) = cubical::write_output(&output, &out_dir) {
                            eprintln!("Transpile write error: {}", err);
                            process::exit(1);
                        }
                        for module in &output.modules {
                            println!(
                                "wrote {}",
                                out_dir
                                    .join(module.path.file_name().unwrap_or_default())
                                    .display()
                            );
                        }
                        println!(
                            "run: cd {} && rustc main.rs && ./main",
                            out_dir.display()
                        );
                    }
                    Err(err) => {
                        eprintln!("Transpile error: {}", err);
                        process::exit(1);
                    }
                }
            } else {
                let file_path = &args[i];
                let path = Path::new(file_path);

                // Auto-detect .pic files and route to cubical mode
                if path.extension().and_then(|s| s.to_str()) == Some("pic") {
                    match cubical::run(file_path) {
                        Ok(output) => println!("{}", output),
                        Err(err) => eprintln!("Cubical error: {}", err),
                    }
                    i += 1;
                    continue;
                }

                // Reject .aot files as direct input
                if path.extension().and_then(|s| s.to_str()) == Some("aot") {
                    eprintln!(
                        "Error: cannot run .aot files directly. Run the matching .pi file:\n  cargo run -- {}",
                        path.with_extension("pi").display()
                    );
                    process::exit(1);
                }

                // Auto-load AOT bytecode if a matching `.aot` file exists
                let aot_path = path.with_extension("aot");
                if aot_path.exists() {
                    match vm::aot_load_file(aot_path.to_str().unwrap()) {
                        Ok(()) => eprintln!("[aot] loaded {}", aot_path.display()),
                        Err(e) => eprintln!("[aot] error loading {}: {}", aot_path.display(), e),
                    }
                }

                let src = match fs::read_to_string(file_path) {
                    Ok(content) => content,
                    Err(err) => {
                        eprintln!(
                            "An error occurred while reading the file '{}': {}",
                            file_path, err
                        );
                        process::exit(1);
                    }
                };
                let base = path.parent();
                with_import_base(base, || run(&src, global_env, &mut heap));

                let (chunks, compilable) = vm::cache_stats();
                eprintln!("[cache] chunks={} compilable={}", chunks, compilable);
            }
            i += 1;
        }
    }
}
