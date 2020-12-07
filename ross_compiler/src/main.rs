#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod cli;
pub mod gen;
pub mod lock;
pub mod parser;

fn main() {
    std::process::exit(match cli::Cli::default().run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            -1
        }
    });
}
