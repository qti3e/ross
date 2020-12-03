#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod parser;

fn main() {
    let source = r#"
    mod documents {
        struct Scope {
            title: str,
        }

        struct Document in Scope as .documents {
            content: str
        }

        action name(x: ref Scope) {
            delete x;
        }
    }

    mod xxx {
        mod t {
            struct Color {}
        }
    }
    "#;

    let declarations = parser::parse(source);
    println!("{:#?}", declarations);
}
