use crate::ast;
mod writer;

pub mod client;

pub trait Backend: Sized {
    fn gen(mut self, root: &ast::Mod) -> String {
        let name = String::from("root");
        self.enter_mod(&name, root);
        self.visit(root);
        self.exit_mod(&name, root);
        self.compile_source()
    }

    fn visit(&mut self, root: &ast::Mod) {
        self.enter_structs();
        for (name, node) in &root.structs {
            self.enter_struct(name, node);
            self.enter_fields(node);
            self.visit_struct_fields(node);
            self.exit_fields(node);
            self.enter_members(node);
            self.visit_struct_members(node);
            self.exit_members(node);
            self.exit_struct(name, node);
        }
        self.exit_structs();

        self.enter_actions();
        for (name, node) in &root.actions {
            self.enter_action(name, node);
            self.enter_parameters(name, node);
            let mut i = 0;
            for (name, ty) in &node.parameters {
                self.action_parameter(name, ty, i);
                i += 1;
            }
            self.exit_parameters(name, node);
            for atom in &node.actions {
                self.action_atom(atom);
            }
            self.exit_action(name, node);
        }
        self.exit_actions();

        self.enter_modules();
        for (name, node) in &root.mods {
            self.enter_mod(name, node);
            self.visit(node);
            self.exit_mod(name, node);
        }
        self.exit_modules();
    }

    #[inline]
    fn visit_struct_fields(&mut self, node: &ast::Struct) {
        for (name, ty) in &node.fields {
            self.struct_field(name, ty);
        }
    }

    #[inline]
    fn visit_struct_members(&mut self, node: &ast::Struct) {
        for (field, object) in &node.members {
            self.struct_member(field, object);
        }
    }

    fn compile_source(self) -> String;

    fn enter_mod(&mut self, _name: &String, _node: &ast::Mod) {}
    fn exit_mod(&mut self, _name: &String, _node: &ast::Mod) {}

    fn enter_struct(&mut self, _name: &String, _node: &ast::Struct) {}
    fn enter_fields(&mut self, _node: &ast::Struct) {}
    fn struct_field(&mut self, _name: &String, _ty: &ast::Type) {}
    fn exit_fields(&mut self, _node: &ast::Struct) {}
    fn enter_members(&mut self, _node: &ast::Struct) {}
    fn struct_member(&mut self, _field: &String, _object: &String) {}
    fn exit_members(&mut self, _node: &ast::Struct) {}
    fn exit_struct(&mut self, _name: &String, _node: &ast::Struct) {}

    fn enter_action(&mut self, _name: &String, _node: &ast::Action) {}
    fn enter_parameters(&mut self, _name: &String, _node: &ast::Action) {}
    fn action_parameter(&mut self, _name: &String, _ty: &ast::Type, _index: usize) {}
    fn exit_parameters(&mut self, _name: &String, _node: &ast::Action) {}
    fn action_atom(&mut self, _atom: &ast::ActionAtom) {}
    fn exit_action(&mut self, _name: &String, _node: &ast::Action) {}

    fn enter_structs(&mut self) {}
    fn exit_structs(&mut self) {}
    fn enter_actions(&mut self) {}
    fn exit_actions(&mut self) {}
    fn enter_modules(&mut self) {}
    fn exit_modules(&mut self) {}
}
