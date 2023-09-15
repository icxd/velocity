use codegen::codegen::{codegen, get_symbols, CodegenState};
use error::Error;
use parsing::{ast::Statement, parser::Parser};
use tokenizer::{token::Token, tokenizer::tokenize};

mod codegen;
mod error;
// mod llvm;
mod parsing;
mod semantic;
mod span;
mod tokenizer;

fn clang_format(filename: &str) -> String {
    let mut binding = std::process::Command::new("clang-format");
    let result = binding
        .arg("--style={BasedOnStyle: llvm, IndentWidth: 4}")
        .arg(filename);
    let result = result.output();
    match result {
        Ok(output) => {
            if output.status.success() {
                return String::from_utf8(output.stdout).unwrap();
            } else {
                println!("Failed to format!");
                println!("{}", String::from_utf8(output.stderr).unwrap());
            }
        }
        Err(error) => println!("{}", error),
    }
    return String::new();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect::<Vec<String>>();
    if args.len() == 0 {
        println!("Usage: velocity <filename>");
        return;
    }

    let filename: String = args[0].clone();
    let contents: String = std::fs::read_to_string(filename.as_str()).unwrap();
    let tokens: Result<Vec<Token>, Error> = tokenize(filename.as_str(), contents.chars());
    match tokens {
        Ok(tokens) => {
            let mut parser: Parser = Parser::new(filename.clone().into(), tokens);
            let ast: Result<Vec<Statement>, Vec<Error>> = parser.parse();
            match ast {
                Ok(ast) => {
                    let only_filename: Vec<&str> = filename.split('/').collect();
                    let only_filename: String = only_filename[only_filename.len() - 1].to_string();
                    let state: &mut CodegenState = &mut CodegenState::new();
                    if let Err(e) = get_symbols(state, &ast) {
                        println!("{}", e);
                        return;
                    }
                    let result: Result<(String, String), Error> =
                        codegen(state, &only_filename, ast);
                    match result {
                        Ok((cpp, hpp)) => {
                            std::fs::write(format!("{}.cpp", filename.clone()), cpp).unwrap();
                            std::fs::write(format!("{}.hpp", filename.clone()), hpp).unwrap();

                            std::fs::write(
                                format!("{}.cpp", filename.clone()),
                                clang_format(format!("{}.cpp", filename.clone()).as_str()),
                            )
                            .unwrap();
                            std::fs::write(
                                format!("{}.hpp", filename.clone()),
                                clang_format(format!("{}.hpp", filename.clone()).as_str()),
                            )
                            .unwrap();

                            let result = std::process::Command::new("clang++")
                                .arg("-std=c++20")
                                .arg(format!("{}.cpp", filename.clone()))
                                .arg("-o")
                                .arg(format!("{}.exe", filename.clone()))
                                .arg("-Iruntime")
                                .output();
                            match result {
                                Ok(output) => {
                                    if output.status.success() {
                                        println!("Compiled successfully!");
                                        let result = std::process::Command::new(format!(
                                            "{}.exe",
                                            filename.clone()
                                        ))
                                        .output();
                                        match result {
                                            Ok(output) => {
                                                if output.status.success() {
                                                    println!(
                                                        "{}",
                                                        String::from_utf8(output.stdout).unwrap()
                                                    );
                                                } else {
                                                    println!("Failed to run!");
                                                    println!(
                                                        "{}",
                                                        String::from_utf8(output.stderr).unwrap()
                                                    );
                                                }
                                            }
                                            Err(error) => println!("{}", error),
                                        }
                                    } else {
                                        println!("Failed to compile!");
                                        println!("{}", String::from_utf8(output.stderr).unwrap());
                                    }
                                }
                                Err(error) => println!("{}", error),
                            }
                        }
                        Err(error) => println!("{}", error),
                    }
                }
                Err(error) => {
                    for error in error {
                        println!("{}", error);
                    }
                }
            }
        }
        Err(error) => println!("{}", error),
    }
}
