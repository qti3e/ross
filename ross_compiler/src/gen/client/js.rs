//! The JavaScript code generator.
//! This module contains the source code for `JavaScriptClientBackend`, which has a
//! self-descriptive name, it is the unit responsible for generating the JavaScript
//! client from a fully-parsed schema, this generator does not generate clean code
//! and tries to minimize the size of the code, one must relay on the TSD generator
//! to generate the TypeScript declaration file which can be used to improve IDE
//! experience by providing auto-completion and type-checking.
//!
//! # Internal Notes
//! This generator relies on these functions that should be defined in the `core`
//! - c(ns, id, name, fields, members): This function generates a class for a struct
//!   and assigns it in the given `namespace` object. (usually $).
//! - p(id, ...Patch[]): Create a BatchAction with the given ID and patch list.
//! - i(Struct): Generate the required patches to insert the given struct.
//! - d(ref): Delete the reference.
//! - s(ref, field_id, new_value): Generate a CAS action.
//! - root._ -> is a map from each struct id to the constructor, the assignment
//!   is donne inn the `c` function and is used for decoding the raw data.

pub use crate::ast;
pub use crate::gen::{writer::Writer, Backend};
use std::fmt::Write;

const CORE_JS: &'static str = include_str!("./core/dist/bundle.js");

pub struct JavaScriptClientBackend {
    w: Writer,
    mod_level: u8,
}

impl JavaScriptClientBackend {
    pub fn new(indention: &str) -> Self {
        let mut w = Writer::new(indention);
        w.write(CORE_JS);
        Self { w, mod_level: 0 }
    }
}

impl Backend for JavaScriptClientBackend {
    fn compile_source(self) -> String {
        self.w.result()
    }

    fn enter_mod(&mut self, name: &String, _: &ast::Mod) {
        if self.mod_level == 0 {
            write!(&mut self.w, "exports.{n} = ($ => {{\n", n = name).unwrap();
            self.w.indent();
            self.w.write("$._ = {};\n"); // Instance ID Map: Map<ID, Constructor>
        } else {
            write!(&mut self.w, "$.{n} = ($ => {{\n", n = name).unwrap();
            self.w.indent();
        }
        self.mod_level += 1;
    }

    fn exit_mod(&mut self, _: &String, _: &ast::Mod) {
        self.mod_level -= 1;
        if self.mod_level == 0 {
            self.w.write("return $;\n");
            self.w.dedent();
            self.w.write("})(Object.create(null));\n");
        } else {
            self.w.write("$.prototype = null;\n");
            self.w.write("return $;\n");
            self.w.dedent();
            self.w.write("})(Object.create($));\n");
        }
    }

    fn enter_struct(&mut self, name: &String, node: &ast::Struct) {
        write!(&mut self.w, "c($, {id}, '{n}', [", n = name, id = node.id).unwrap();
    }

    fn struct_field(&mut self, name: &String, ty: &ast::Type) {
        match ty {
            ast::Type::Object(obj) => write!(&mut self.w, "['{n}', $.{o}], ", n = name, o = obj),
            ast::Type::ObjectRef(_) => write!(&mut self.w, "['{n}'], ", n = name),
            _ => write!(&mut self.w, "'{n}', ", n = name),
        }
        .unwrap();
    }

    fn exit_fields(&mut self, _node: &ast::Struct) {
        self.w.write("], [")
    }

    fn struct_member(&mut self, field: &String, _object: &String) {
        write!(&mut self.w, "'{n}', ", n = field).unwrap()
    }

    fn exit_struct(&mut self, _name: &String, node: &ast::Struct) {
        if let Some((_, field)) = &node.owner {
            write!(&mut self.w, "], '{n}');\n", n = field).unwrap()
        } else {
            self.w.write("]);\n");
        }
    }

    fn enter_actions(&mut self) {
        self.w
            .write("const $$ = $.actions = Object.create(null);\n")
    }

    fn enter_action(&mut self, name: &String, _node: &ast::Action) {
        write!(&mut self.w, "$$.{n} = (", n = name).unwrap();
    }

    fn action_parameter(&mut self, name: &String, _: &ast::Type, index: usize) {
        if index > 0 {
            write!(&mut self.w, ", {}", name).unwrap();
        } else {
            self.w.write(name.as_str());
        }
    }

    fn exit_parameters(&mut self, _: &String, node: &ast::Action) {
        write!(&mut self.w, ") => p({},\n", node.id).unwrap();
        self.w.indent();
    }

    fn exit_action(&mut self, _name: &String, _node: &ast::Action) {
        self.w.dedent();
        self.w.write(");\n");
    }

    fn action_atom(&mut self, atom: &ast::ActionAtom) {
        match atom {
            ast::ActionAtom::Insert { parameter, .. } => {
                write!(&mut self.w, "i({}),\n", parameter).unwrap();
            }
            ast::ActionAtom::Delete { parameter, .. } => {
                write!(&mut self.w, "d({}),\n", parameter).unwrap();
            }
        }
    }
}
