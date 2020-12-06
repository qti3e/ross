#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod gen;
pub mod lock;
pub mod parser;

use gen::Backend;

fn main() {
    let source = r#"
    struct Point {
        line: num,
        column: num
    }

    struct Range {
        from: Point,
        to: Point
    }
    
    mod colors {
        struct Color {
            r: num,
            g: num,
            b: num
        }

        struct Space {
            title: str
        }

        struct Shape in Space as .shapes {
            color: Color,
            size: num
        }

        action insertShape(shape: Shape) {
            insert shape;
        }
    }
    "#;

    let root = parser::parse(source).unwrap();
    let lock = lock::Lock::from(&root);
    println!("{:#?}", root);
    println!("{:#?}", lock);
    let jsb = gen::client::js::JavaScriptClientBackend::new("  ");
    let js = jsb.gen(&root);
    println!("js:\n\n{}", js);
}
