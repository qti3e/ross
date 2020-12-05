#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod lock;
pub mod parser;

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
    "#;

    let root = parser::parse(source).unwrap();
    let lock = lock::Lock::from(&root);
    println!("{:#?}", root);
    println!("{:#?}", lock);
}
