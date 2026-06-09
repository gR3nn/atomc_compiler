pub mod lexer;
pub mod parser;
pub mod symbol;
pub mod symtable;
pub mod token;

use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use token::TokenCode;

fn print_help() {
    println!("AtomC Compiler");
    println!("Usage:");
    println!("  atomc -h                Show this help message");
    println!(
        "  atomc -test             Run the lexer against all .c files in the tests/ directory"
    );
    println!("  atomc <filename>        Compile the specified source file");
}

fn compile_file(filepath: &str) {
    println!("--Compiling file!--");

    let source = match fs::read_to_string(filepath) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}' : {}", filepath, e);
            std::process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();

    //LEXICAL ANALYSIS
    loop {
        let token = lexer.get_next_token();

        //commented out to not flood the terminal
        //println!("Line {:03}: {:?}", token.line, token.code);

        let is_end = token.code == TokenCode::END;
        tokens.push(token);

        if is_end {
            break;
        }
    }

    println!("Finished Lexing {}", filepath);

    //SYNTACTIC ANALYSIS
    println!("Starting Syntax Analysis for {}", filepath);
    let mut parser = Parser::new(tokens);
    parser.parse(); // run the parser

    // if parser.parse() finishes without calling self.err(), the syntax is correct
    println!("Syntax Analysis Successful for {}\n", filepath);
}

fn run_tests() {
    let test_dir = "tests";
    let path = Path::new(test_dir);

    if !path.exists() || !path.is_dir() {
        eprintln!(
            "Error: '{}' directory not found. Please create it and add your test files.",
            test_dir
        );
        std::process::exit(1);
    }

    println!("Running integration tests from '{}'...", test_dir);

    let mut files: Vec<_> = fs::read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|d| d.path().extension().and_then(|s| s.to_str()) == Some("c"))
        .collect();

    files.sort_by_key(|dir| dir.path());

    for entry in files {
        let filepath = entry.path();
        compile_file(filepath.to_str().unwrap());
    }
}

fn main() {
    // Collect command line arguments
    let args: Vec<String> = env::args().collect();

    // If no arguments are provided, show help and exit
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    // Match the first provided argument
    match args[1].as_str() {
        "-h" | "--help" => print_help(),
        "-test" => run_tests(),
        filename => compile_file(filename),
    }
}
