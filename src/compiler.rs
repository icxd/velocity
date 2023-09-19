use std::rc::Rc;

use crate::{error::Error, parser::Parser, tokenizer::Tokenizer};

pub(crate) struct Compiler {
    files: Vec<String>,
}

impl Compiler {
    pub(crate) fn new() -> Compiler {
        Compiler { files: Vec::new() }
    }

    pub(crate) fn add_file(&mut self, filename: String) {
        self.files.push(filename);
    }

    pub(crate) fn compile(&self) -> Result<(), Error> {
        for filename in &self.files {
            let contents: String = match std::fs::read_to_string(filename.as_str()) {
                Ok(contents) => contents,
                Err(_) => {
                    return Err(Error::new(
                        format!("failed to read file '{}'", filename).as_str(),
                        (Rc::new("<stdin>".to_string()), (0, 0)..(0, 0)),
                    ))
                }
            };
            let mut tokenizer = Tokenizer::new(Rc::new(filename.clone()), contents);
            let tokens = match tokenizer.tokenize() {
                Ok(tokens) => tokens,
                Err(err) => return Err(err),
            };

            let mut parser = Parser::new(tokens);
            let statements = match parser.parse() {
                Ok(statements) => statements,
                Err(err) => return Err(err),
            };

            for statement in statements {
                println!("{:?}", statement);
            }
        }
        Ok(())
    }
}
