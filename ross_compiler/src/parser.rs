use crate::ast::{
    self,
    builder::{ASTBuilder, BuilderError},
};
use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "ross.pest"]
struct RossParser;

pub fn parse(source: &str) -> Result<ast::Mod, BuilderError> {
    let mut builder = ASTBuilder::new();

    let pairs = RossParser::parse(Rule::program, source).unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        visit_declaration(&mut builder, pair)?;
    }

    builder.finalize()
}

fn visit_declaration(builder: &mut ASTBuilder, pair: Pair<Rule>) -> Result<(), BuilderError> {
    match pair.as_rule() {
        Rule::mod_declaration => {
            builder.enter_mod()?;

            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::mod_name => {
                        builder.name(pair.as_str().into())?;
                    }
                    _ => {
                        visit_declaration(builder, pair)?;
                    }
                }
            }

            builder.exit_mod()?;
        }
        Rule::struct_declaration => {
            builder.enter_struct()?;

            let mut owner_name = None;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::struct_name => {
                        builder.name(pair.as_str().into())?;
                    }
                    Rule::struct_field_name => {
                        builder.field_name(pair.as_str().into())?;
                    }
                    Rule::struct_field_type => {
                        let ty_pair = pair.into_inner().peek().unwrap();
                        let ty = resolve_type(&builder, ty_pair)?;
                        builder.field_type(ty)?;
                    }
                    Rule::owner_name => {
                        owner_name = Some(pair.as_str());
                    }
                    Rule::owner_field_name => {
                        builder.owner(owner_name.unwrap(), pair.as_str())?;
                    }
                    _ => unreachable!(),
                }
            }

            builder.exit_struct()?;
        }
        Rule::action_declaration => {
            builder.enter_action()?;

            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::action_name => {
                        builder.name(pair.as_str().into())?;
                    }
                    Rule::parameter_name => {
                        builder.parameter_name(pair.as_str().into())?;
                    }
                    Rule::parameter_type => {
                        let ty_pair = pair.into_inner().peek().unwrap();
                        let ty = resolve_type(&builder, ty_pair)?;
                        builder.parameter_type(ty)?;
                    }
                    Rule::insert_action => {
                        let name = pair.into_inner().peek().unwrap();
                        builder.insert(name.as_str())?;
                    }
                    Rule::delete_action => {
                        let name = pair.into_inner().peek().unwrap();
                        builder.delete(name.as_str())?;
                    }
                    _ => {
                        println!("Ac > {:?}", pair);
                    }
                }
            }

            builder.exit_action()?;
        }
        Rule::EOI => {}
        _ => unreachable!(),
    }

    Ok(())
}

fn resolve_type(builder: &ASTBuilder, pair: Pair<Rule>) -> Result<ast::Type, BuilderError> {
    match pair.as_rule() {
        Rule::primitive_type => Ok(match pair.as_str() {
            "bool" => ast::Type::Bool,
            "str" => ast::Type::Str,
            "num" => ast::Type::Num,
            "hash" => ast::Type::Hash,
            _ => unreachable!(),
        }),
        Rule::object_type => builder.resolve_obj(pair.as_str(), false),
        Rule::ref_type => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::object_type => return builder.resolve_obj(pair.as_str(), true),
                    _ => {}
                }
            }
            unreachable!()
        }
        _ => unreachable!(),
    }
}
