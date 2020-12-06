pub use crate::ast;
pub use crate::gen::{writer::Writer, Backend};
use std::fmt::Write;

pub struct JavaScriptClientBackend {
    w: Writer,
    mod_level: u8,
}

impl JavaScriptClientBackend {
    pub fn new(indention: &str) -> Self {
        Self {
            w: Writer::new(indention),
            mod_level: 0,
        }
    }
}

impl Backend for JavaScriptClientBackend {
    fn compile_source(self) -> String {
        self.w.result()
    }

    fn enter_mod(&mut self, name: &String, _: &ast::Mod) {
        if self.mod_level == 0 {
            write!(&mut self.w, "const {n} = (($, _) => {{\n", n = name)
        } else {
            write!(&mut self.w, "$.{n} = ($ => {{\n", n = name)
        }
        .unwrap();
        self.w.indent();
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

    fn enter_struct(&mut self, name: &String, _node: &ast::Struct) {
        write!(&mut self.w, "_ = '{n}';$[_] = gc(_, [", n = name).unwrap();
    }

    fn struct_field(&mut self, name: &String, ty: &ast::Type) {
        match ty {
            ast::Type::Object(obj) => write!(&mut self.w, "['{n}', $.{o}.$], ", n = name, o = obj),
            _ => write!(&mut self.w, "'{n}', ", n = name),
        }
        .unwrap();
    }

    fn exit_fields(&mut self, node: &ast::Struct) {
        if node.members.len() > 0 {
            self.w.write("], [")
        } else {
            self.w.write("]);\n")
        }
    }

    fn struct_member(&mut self, field: &String, _object: &String) {
        write!(&mut self.w, "'{n}', ", n = field).unwrap()
    }

    fn exit_struct(&mut self, _name: &String, node: &ast::Struct) {
        if node.members.len() > 0 {
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

    fn exit_parameters(&mut self, _: &String, _: &ast::Action) {
        self.w.write(") => [].concat(\n");
        self.w.indent();
    }

    fn exit_action(&mut self, _name: &String, _node: &ast::Action) {
        self.w.dedent();
        self.w.write(");\n");
    }

    fn action_atom(&mut self, atom: &ast::ActionAtom) {
        match atom {
            ast::ActionAtom::Insert { parameter, .. } => {
                write!(&mut self.w, "insert({}),\n", parameter).unwrap();
            }
            ast::ActionAtom::Delete { parameter, .. } => {
                write!(&mut self.w, "delete({}),\n", parameter).unwrap();
            }
        }
    }
}
