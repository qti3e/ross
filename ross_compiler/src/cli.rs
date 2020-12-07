use crate::ast;
use crate::gen::{self, Backend};
use crate::parser;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs;
use std::path::Path;

pub struct Cli {}

impl Default for Cli {
    fn default() -> Self {
        Cli {}
    }
}

impl Cli {
    fn build_app() -> clap::App<'static, 'static> {
        App::new("Ross Compiler")
            .version("0.1.0")
            .author("Parsa G. <me@qti3e.com>")
            .about("Ross Schema Parser & Code Generator.")
            .subcommands(vec![
                SubCommand::with_name("check")
                    .about("Validate the schema.")
                    .arg(
                        Arg::with_name("INPUT")
                            .help("Sets the input file to use.")
                            .required(true),
                    ),
                SubCommand::with_name("gen")
                    .about("Generate the source documents from the schema.")
                    .arg(
                        Arg::with_name("INPUT")
                            .help("Sets the input file to use.")
                            .required(true),
                    )
                    .arg(
                        Arg::with_name("OUTDIR")
                            .help("Output directory to write the generated codes.")
                            .required(true),
                    ),
                SubCommand::with_name("ast")
                    .about("Prints the AST of the source file.")
                    .arg(
                        Arg::with_name("INPUT")
                            .help("Sets the input file to use.")
                            .required(true),
                    ),
            ])
    }

    /// Run the CLI app, returns false in case of failure.
    pub fn run(self) -> Result<(), String> {
        let app_matches = Self::build_app().get_matches();
        match app_matches.subcommand() {
            ("ast", Some(sub)) => {
                let ast = Cli::open(sub)?;
                println!("{:#?}", ast);
                Ok(())
            }
            ("check", Some(sub)) => {
                Cli::open(sub)?;
                // TODO(qti3e) Check lock.
                Ok(())
            }
            ("gen", Some(sub)) => {
                let ast = Cli::open(sub)?;
                // TODO(qti3e) Check lock.
                Cli::write(ast, sub)?;
                Ok(())
            }
            _ => Err(format!(
                "{}\n Use --help for more info.",
                app_matches.usage()
            )),
        }
    }

    fn open(sub: &ArgMatches) -> Result<ast::Mod, String> {
        let input = sub.value_of("INPUT").unwrap().to_string();
        let path = Path::new(&input);
        let source =
            fs::read_to_string(path).map_err(|e| format!("Cannot read the input: {}", e))?;
        parser::parse(&source).map_err(|e| format!("Parse error: {}", e))
    }

    fn write(ast: ast::Mod, sub: &ArgMatches) -> Result<(), String> {
        let js = gen::client::tsd::TypeScriptClientBackend::new("    ");
        let source = js.gen(&ast);
        println!("{}", source);
        Ok(())
    }
}
