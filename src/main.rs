use error::Error;
use parsing::{ast::Statement, parser::Parser};
use tokenizer::{token::Token, tokenizer::tokenize};

mod error;
mod parsing;
mod span;
mod tokenizer;

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
            let mut parser: Parser = Parser::new(filename.into(), tokens);
            let ast: Result<Vec<Statement>, Vec<Error>> = parser.parse();
            match ast {
                Ok(ast) => {
                    for statement in ast {
                        println!("{:?}", statement);
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
