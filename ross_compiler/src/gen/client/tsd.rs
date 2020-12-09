pub use crate::ast;
pub use crate::gen::{writer::Writer, Backend};
use std::fmt::Write;

const CORE_TS: &'static str = include_str!("./core/dist/bundle.d.ts");

pub struct TypeScriptClientBackend {
    w: Writer,
    mod_level: u8,
    in_constructor: bool,
}

impl TypeScriptClientBackend {
    pub fn new(indention: &str) -> Self {
        let mut w = Writer::new(indention);
        w.write(CORE_TS);
        Self {
            w,
            mod_level: 0,
            in_constructor: false,
        }
    }
}

impl Backend for TypeScriptClientBackend {
    fn compile_source(self) -> String {
        self.w.result()
    }

    fn enter_mod(&mut self, name: &String, _: &ast::Mod) {
        if self.mod_level == 0 {
            write!(&mut self.w, "export declare namespace {n} {{\n", n = name).unwrap();
            self.w.indent();
            self.w.write("export const _: Record<number, StructConstructor>;\n"); // Instance ID Map: Map<ID, Constructor>
        } else {
            write!(&mut self.w, "export namespace {n} {{\n", n = name).unwrap();
            self.w.indent();
        }
        self.mod_level += 1;
    }

    fn exit_mod(&mut self, _: &String, _: &ast::Mod) {
        self.mod_level -= 1;
        self.w.dedent();
        self.w.write("}\n");
    }

    fn enter_struct(&mut self, name: &String, _node: &ast::Struct) {
        self.in_constructor = false;
        write!(&mut self.w, "export class {n} extends RossStruct {{\n", n = name).unwrap();
        self.w.indent();
        self.w.write("getAllChildren(): RossStruct[];\n");
        self.w.write("getPathFor(fieldId: number): string[];\n");
        self.w.write("encode(): ObjectRawData;\n");
        self.w.write("static readonly $: Field[];\n")
    }

    fn struct_field(&mut self, name: &String, ty: &ast::Type) {
        let ty = match ty {
            ast::Type::Object(obj) => format!("{n}: {o}", n = name, o = obj),
            ast::Type::ObjectRef(obj) => format!("{n}: Ref<{o}>", n = name, o = obj),
            ast::Type::Primitive(p) => match p {
                ast::PrimitiveType::Null => format!("{n}: null", n = name),
                ast::PrimitiveType::Bool => format!("{n}: boolean", n = name),
                ast::PrimitiveType::Hash => format!("{n}: Hash16", n = name),
                ast::PrimitiveType::Num => format!("{n}: number", n = name),
                ast::PrimitiveType::Str => format!("{n}: string", n = name),
            },
        };

        if self.in_constructor {
            write!(&mut self.w, "{},\n", ty).unwrap();
        } else {
            write!(&mut self.w, "readonly {};\n", ty).unwrap();
        }
    }

    fn struct_member(&mut self, field: &String, object: &String) {
        write!(
            &mut self.w,
            "readonly {n}: Ref<{o}>[];\n",
            n = field,
            o = object
        )
        .unwrap()
    }

    fn exit_struct(&mut self, _name: &String, node: &ast::Struct) {
        self.w.write("constructor(\n");
        self.w.indent();
        self.in_constructor = true;
        self.visit_struct_fields(node);
        self.w.dedent();
        self.w.write(");\n");

        self.w.dedent();
        self.w.write("}\n");
    }

    fn enter_actions(&mut self) {
        self.w.write("export namespace actions {\n");
        self.w.indent();
    }

    fn exit_actions(&mut self) {
        self.w.dedent();
        self.w.write("}\n");
    }

    fn enter_action(&mut self, name: &String, _node: &ast::Action) {
        write!(&mut self.w, "export function {n}(", n = name).unwrap();
    }

    fn action_parameter(&mut self, name: &String, ty: &ast::Type, index: usize) {
        let t = match ty {
            ast::Type::Object(obj) => obj.clone(),
            ast::Type::ObjectRef(obj) => format!("Ref<{}>", obj),
            ast::Type::Primitive(p) => match p {
                ast::PrimitiveType::Null => "null",
                ast::PrimitiveType::Bool => "boolean",
                ast::PrimitiveType::Hash => "Hash16",
                ast::PrimitiveType::Num => "number",
                ast::PrimitiveType::Str => "string",
            }
            .to_string(),
        };
        if index > 0 {
            write!(&mut self.w, ", {n}: {t}", n = name, t = t).unwrap();
        } else {
            write!(&mut self.w, "{n}: {t}", n = name, t = t).unwrap();
        }
    }

    fn exit_parameters(&mut self, _: &String, _: &ast::Action) {
        write!(&mut self.w, "): BatchPatch;\n").unwrap();
    }
}
