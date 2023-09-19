use compiler::Compiler;

mod ast;
mod compiler;
mod error;
mod parser;
mod span;
mod tokenizer;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect::<Vec<String>>();
    if args.len() == 0 {
        println!("Usage: velocity <filename>");
        return;
    }

    let mut compiler = Compiler::new();
    for filename in args {
        compiler.add_file(filename);
    }

    match compiler.compile() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    }
}
