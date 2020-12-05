use indexmap::IndexMap;

#[derive(Debug)]
pub struct Mod {
    pub structs: IndexMap<String, Struct>,
    pub actions: IndexMap<String, Action>,
    pub mods: IndexMap<String, Mod>,
}

#[derive(Debug)]
pub struct Struct {
    pub id: u32,
    pub owner: Option<(String, String)>,
    pub fields: IndexMap<String, Type>,
    pub members: IndexMap<String, String>,
    pub type_vec: Vec<PrimitiveType>,
}

#[derive(Debug)]
pub struct Action {
    pub id: u32,
    pub parameters: IndexMap<String, Type>,
    pub actions: Vec<ActionAtom>,
}

#[derive(Debug)]
pub enum ActionAtom {
    Insert {
        parameter_index: usize,
        parameter: String,
        ty: String,
    },
    Delete {
        parameter_index: usize,
        parameter: String,
        ty: String,
    },
}

#[derive(Debug)]
pub enum Type {
    Object(String),
    ObjectRef(String),
    Primitive(PrimitiveType),
}

#[derive(Debug, Copy, Clone)]
pub enum PrimitiveType {
    Bool,
    Str,
    Num,
    Hash,
}

pub mod builder {
    use super::*;
    use indexmap::IndexMap;
    use std::fmt;

    pub struct ASTBuilder {
        frames: Vec<State>,
        state: State,
        path: Vec<usize>,
    }

    #[derive(Debug)]
    pub enum BuilderError {
        OperationOnInvalidState,
        MissingModName,
        MissingStructName,
        MissingActionName,
        NameAlreadyInUse(String),
        FieldNotComplete(String),
        CanNotResolveName(String),
        UnexpectedEnd,
        IDOutOfBound(Vec<usize>, usize),
        InsertTypeError,
        DeleteTypeError,
    }

    impl std::error::Error for BuilderError {}

    impl fmt::Display for BuilderError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                BuilderError::OperationOnInvalidState => write!(
                    f,
                    "The method which was called is not valid in the current state."
                ),
                BuilderError::MissingModName => write!(f, "Module name was missing."),
                BuilderError::MissingStructName => write!(f, "Struct name was missing."),
                BuilderError::MissingActionName => write!(f, "Action name was missing."),
                BuilderError::NameAlreadyInUse(name) => {
                    write!(f, "Name '{}' is already in use.", name)
                }
                BuilderError::FieldNotComplete(name) => {
                    write!(f, "Field '{}' was provided with no type.", name)
                }
                BuilderError::CanNotResolveName(name) => {
                    write!(f, "Cannot resolve name '{}'", name)
                }
                BuilderError::UnexpectedEnd => {
                    write!(f, "The AST-builder was terminated before completion.")
                }
                BuilderError::IDOutOfBound(v, i) => write!(
                    f,
                    "Could not pack the ID into an u32. (path={:?}, id={}.)",
                    v, i
                ),
                BuilderError::InsertTypeError => {
                    write!(f, "Insert statement only accepts owned objects (not ref.)")
                }
                BuilderError::DeleteTypeError => {
                    write!(f, "Delete statement only accepts referenced objects.")
                }
            }
        }
    }

    enum State {
        Mod {
            name: Option<String>,
            structs: IndexMap<String, Struct>,
            actions: IndexMap<String, Action>,
            mods: IndexMap<String, Mod>,
        },
        Struct {
            name: Option<String>,
            id: u32,
            owner: Option<(String, String)>,
            fields: IndexMap<String, Type>,
            field_name: Option<String>,
            field_type: Option<Type>,
        },
        Action {
            name: Option<String>,
            id: u32,
            parameters: IndexMap<String, Type>,
            actions: Vec<ActionAtom>,
            parameter_name: Option<String>,
            parameter_type: Option<Type>,
        },
    }

    impl ASTBuilder {
        pub fn new() -> Self {
            ASTBuilder {
                frames: Vec::with_capacity(5),
                path: Vec::with_capacity(5),
                state: State::Mod {
                    name: None,
                    structs: IndexMap::new(),
                    actions: IndexMap::new(),
                    mods: IndexMap::new(),
                },
            }
        }

        pub fn name(&mut self, n: String) -> Result<(), BuilderError> {
            let name_used = match &mut self.state {
                State::Mod { name, .. } => {
                    *name = Some(n.clone());
                    self.frames.last().map(|f| match f {
                        State::Mod { mods, .. } => mods.contains_key(&n),
                        _ => unreachable!(),
                    })
                }
                State::Struct { name, .. } => {
                    *name = Some(n.clone());
                    self.frames.last().map(|f| match f {
                        State::Mod { structs, .. } => structs.contains_key(&n),
                        _ => unreachable!(),
                    })
                }
                State::Action { name, .. } => {
                    *name = Some(n.clone());
                    self.frames.last().map(|f| match f {
                        State::Mod { actions, .. } => actions.contains_key(&n),
                        _ => unreachable!(),
                    })
                }
            };

            if name_used.unwrap() {
                return Err(BuilderError::NameAlreadyInUse(n));
            }

            Ok(())
        }

        pub fn enter_mod(&mut self) -> Result<(), BuilderError> {
            let index = match &self.state {
                State::Mod { mods, .. } => mods.len(),
                _ => return Err(BuilderError::OperationOnInvalidState),
            };

            let mut next_state = State::Mod {
                name: None,
                structs: IndexMap::new(),
                actions: IndexMap::new(),
                mods: IndexMap::new(),
            };

            std::mem::swap(&mut next_state, &mut self.state);
            // state = self.state
            // self.state = next state
            self.frames.push(next_state);
            self.path.push(index);

            Ok(())
        }

        pub fn exit_mod(&mut self) -> Result<(), BuilderError> {
            let mut state = self.frames.pop().unwrap();
            std::mem::swap(&mut state, &mut self.state);

            let (name, declaration) = match state {
                State::Mod {
                    name,
                    mods,
                    structs,
                    actions,
                    ..
                } => (
                    name.ok_or(BuilderError::MissingModName)?,
                    Mod {
                        mods,
                        structs,
                        actions,
                    },
                ),
                mut state => {
                    std::mem::swap(&mut state, &mut self.state);
                    self.frames.push(state);
                    return Err(BuilderError::OperationOnInvalidState);
                }
            };

            match &mut self.state {
                State::Mod { mods, .. } => {
                    mods.insert(name, declaration);
                    self.path.pop().unwrap();
                    Ok(())
                }
                _ => unreachable!(),
            }
        }

        pub fn enter_struct(&mut self) -> Result<(), BuilderError> {
            let index = match &self.state {
                State::Mod { structs, .. } => structs.len(),
                _ => return Err(BuilderError::OperationOnInvalidState),
            };

            let id = pack_id(&self.path, index)
                .ok_or_else(|| BuilderError::IDOutOfBound(self.path.clone(), index))?;
            let mut next_state = State::Struct {
                name: None,
                id,
                owner: None,
                fields: IndexMap::new(),
                field_name: None,
                field_type: None,
            };
            std::mem::swap(&mut next_state, &mut self.state);
            self.frames.push(next_state);

            Ok(())
        }

        pub fn exit_struct(&mut self) -> Result<(), BuilderError> {
            let mut state = self.frames.pop().unwrap();
            std::mem::swap(&mut state, &mut self.state);

            let (name, mut declaration) = match state {
                State::Struct {
                    name,
                    id,
                    owner,
                    fields,
                    ..
                } => (
                    name.ok_or(BuilderError::MissingStructName)?,
                    Struct {
                        owner,
                        id,
                        fields,
                        members: IndexMap::new(),
                        type_vec: Vec::new(),
                    },
                ),
                mut state => {
                    std::mem::swap(&mut state, &mut self.state);
                    self.frames.push(state);
                    return Err(BuilderError::OperationOnInvalidState);
                }
            };

            match &mut self.state {
                State::Mod { structs, .. } => {
                    collect_type_vec(&structs, &declaration.fields, &mut declaration.type_vec);
                    structs.insert(name, declaration);
                    Ok(())
                }
                _ => unreachable!(),
            }
        }

        fn field_finalize(&mut self) {
            match &mut self.state {
                State::Struct {
                    fields,
                    field_name,
                    field_type,
                    ..
                } if field_name.is_some() && field_type.is_some() => {
                    fields.insert(field_name.take().unwrap(), field_type.take().unwrap());
                }
                State::Struct { .. } => {}
                _ => unreachable!(),
            }
        }

        pub fn field_name(&mut self, name: String) -> Result<(), BuilderError> {
            match &mut self.state {
                State::Struct { field_name, .. } if field_name.is_some() => {
                    Err(BuilderError::FieldNotComplete(field_name.take().unwrap()))
                }
                State::Struct {
                    field_name, fields, ..
                } => {
                    if fields.contains_key(&name) {
                        Err(BuilderError::NameAlreadyInUse(name))
                    } else {
                        field_name.replace(name);
                        self.field_finalize();
                        Ok(())
                    }
                }
                _ => Err(BuilderError::OperationOnInvalidState),
            }
        }

        pub fn field_type(&mut self, ty: Type) -> Result<(), BuilderError> {
            match &mut self.state {
                State::Struct { field_type, .. } => {
                    field_type.replace(ty);
                    self.field_finalize();
                    Ok(())
                }
                _ => Err(BuilderError::OperationOnInvalidState),
            }
        }

        pub fn owner(&mut self, struct_name: &str, field: &str) -> Result<(), BuilderError> {
            let name = match &mut self.state {
                State::Struct { owner, name, .. } => {
                    owner.replace((struct_name.into(), field.into()));
                    name.clone().unwrap()
                }
                _ => return Err(BuilderError::OperationOnInvalidState),
            };

            let st = self.find_struct_mut(struct_name)?;
            if st.fields.contains_key(field) || st.members.contains_key(field) {
                return Err(BuilderError::NameAlreadyInUse(field.into()));
            }
            st.members.insert(field.into(), name);

            Ok(())
        }

        pub fn enter_action(&mut self) -> Result<(), BuilderError> {
            let index = match &self.state {
                State::Mod { actions, .. } => actions.len(),
                _ => return Err(BuilderError::OperationOnInvalidState),
            };

            let id = pack_id(&self.path, index)
                .ok_or_else(|| BuilderError::IDOutOfBound(self.path.clone(), index))?;
            let mut next_state = State::Action {
                name: None,
                id,
                parameters: IndexMap::new(),
                actions: vec![],
                parameter_name: None,
                parameter_type: None,
            };
            std::mem::swap(&mut next_state, &mut self.state);
            self.frames.push(next_state);

            Ok(())
        }

        pub fn exit_action(&mut self) -> Result<(), BuilderError> {
            let mut state = self.frames.pop().unwrap();
            std::mem::swap(&mut state, &mut self.state);

            let (name, declaration) = match state {
                State::Action {
                    name,
                    id,
                    parameters,
                    actions,
                    ..
                } => (
                    name.ok_or(BuilderError::MissingActionName)?,
                    Action {
                        id,
                        parameters,
                        actions,
                    },
                ),
                mut state => {
                    std::mem::swap(&mut state, &mut self.state);
                    self.frames.push(state);
                    return Err(BuilderError::OperationOnInvalidState);
                }
            };

            match &mut self.state {
                State::Mod { actions, .. } => {
                    actions.insert(name, declaration);
                    Ok(())
                }
                _ => unreachable!(),
            }
        }

        fn parameter_finalize(&mut self) {
            match &mut self.state {
                State::Action {
                    parameters,
                    parameter_name,
                    parameter_type,
                    ..
                } if parameter_name.is_some() && parameter_type.is_some() => {
                    parameters.insert(
                        parameter_name.take().unwrap(),
                        parameter_type.take().unwrap(),
                    );
                }
                State::Action { .. } => {}
                _ => unreachable!(),
            }
        }

        pub fn parameter_name(&mut self, name: String) -> Result<(), BuilderError> {
            match &mut self.state {
                State::Action { parameter_name, .. } if parameter_name.is_some() => Err(
                    BuilderError::FieldNotComplete(parameter_name.take().unwrap()),
                ),
                State::Action {
                    parameter_name,
                    parameters,
                    ..
                } => {
                    if parameters.contains_key(&name) {
                        Err(BuilderError::NameAlreadyInUse(name))
                    } else {
                        parameter_name.replace(name);
                        self.parameter_finalize();
                        Ok(())
                    }
                }
                _ => Err(BuilderError::OperationOnInvalidState),
            }
        }

        pub fn parameter_type(&mut self, ty: Type) -> Result<(), BuilderError> {
            match &mut self.state {
                State::Action { parameter_type, .. } => {
                    parameter_type.replace(ty);
                    self.parameter_finalize();
                    Ok(())
                }
                _ => Err(BuilderError::OperationOnInvalidState),
            }
        }

        pub fn insert(&mut self, parameter_name: &str) -> Result<(), BuilderError> {
            let (parameter, ty) = self.resolve_parameter(parameter_name)?;

            let action = match ty {
                Type::Object(o) => ActionAtom::Insert {
                    parameter_index: parameter,
                    parameter: parameter_name.into(),
                    ty: o.clone(),
                },
                _ => return Err(BuilderError::InsertTypeError),
            };

            match &mut self.state {
                State::Action { actions, .. } => {
                    actions.push(action);
                    Ok(())
                }
                _ => unreachable!(),
            }
        }

        pub fn delete(&mut self, parameter_name: &str) -> Result<(), BuilderError> {
            let (parameter, ty) = self.resolve_parameter(parameter_name)?;

            let action = match ty {
                Type::ObjectRef(o) => ActionAtom::Delete {
                    parameter_index: parameter,
                    parameter: parameter_name.into(),
                    ty: o.clone(),
                },
                _ => return Err(BuilderError::DeleteTypeError),
            };

            match &mut self.state {
                State::Action { actions, .. } => {
                    actions.push(action);
                    Ok(())
                }
                _ => unreachable!(),
            }
        }

        pub fn resolve_parameter(&self, n: &str) -> Result<(usize, &Type), BuilderError> {
            match &self.state {
                State::Action { parameters, .. } => parameters
                    .get_full(n)
                    .map(|(i, _, t)| (i, t))
                    .ok_or_else(|| BuilderError::CanNotResolveName(n.into())),
                _ => Err(BuilderError::OperationOnInvalidState),
            }
        }

        fn find_struct_mut(&mut self, n: &str) -> Result<&mut Struct, BuilderError> {
            match self.last_mod_mut() {
                State::Mod { structs, .. } => structs
                    .get_mut(n)
                    .ok_or_else(|| BuilderError::CanNotResolveName(n.into())),
                _ => unreachable!(),
            }
        }

        fn last_mod_mut(&mut self) -> &mut State {
            match &self.state {
                State::Mod { .. } => return &mut self.state,
                _ => {}
            };

            for s in self.frames.iter_mut().rev() {
                match &s {
                    State::Mod { .. } => return s,
                    _ => {}
                };
            }

            unreachable!()
        }

        fn last_mod(&self) -> &State {
            match &self.state {
                State::Mod { .. } => return &self.state,
                _ => {}
            };

            for s in self.frames.iter().rev() {
                match &s {
                    State::Mod { .. } => return &s,
                    _ => {}
                };
            }

            unreachable!()
        }

        pub fn resolve_obj(&self, name: &str, is_ref: bool) -> Result<Type, BuilderError> {
            match self.last_mod() {
                State::Mod { structs, .. } => {
                    if structs.contains_key(name) {
                        Ok(if is_ref {
                            Type::ObjectRef(name.into())
                        } else {
                            Type::Object(name.into())
                        })
                    } else {
                        Err(BuilderError::CanNotResolveName(name.into()))
                    }
                }
                _ => unreachable!(),
            }
        }

        pub fn finalize(self) -> Result<Mod, BuilderError> {
            if self.frames.len() > 0 {
                return Err(BuilderError::UnexpectedEnd);
            }

            match self.state {
                State::Mod {
                    actions,
                    structs,
                    mods,
                    ..
                } => Ok(Mod {
                    actions,
                    structs,
                    mods,
                }),
                _ => unreachable!(),
            }
        }
    }

    #[inline]
    fn pack_id(path: &Vec<usize>, id: usize) -> Option<u32> {
        if path.len() > 4 {
            return None;
        }

        let mut parts = [0u32; 5];
        for (i, p) in path.iter().enumerate() {
            if (p + 1) > 63 {
                return None;
            }
            parts[i] = (p + 1) as u32;
        }
        if id > 255 {
            return None;
        }
        parts[4] = id as u32;

        let mut ret = 0u32;
        ret |= parts[4];
        ret |= parts[0] << 8;
        ret |= parts[1] << 14;
        ret |= parts[2] << 20;
        ret |= parts[3] << 26;

        Some(ret)
    }

    #[inline]
    fn collect_type_vec(
        structs: &IndexMap<String, Struct>,
        fields: &IndexMap<String, Type>,
        mut type_vec: &mut Vec<PrimitiveType>,
    ) {
        fn visit(
            structs: &IndexMap<String, Struct>,
            mut vec: &mut Vec<PrimitiveType>,
            fields: &IndexMap<String, Type>,
        ) {
            for (_, ty) in fields {
                match ty {
                    Type::Primitive(t) => {
                        vec.push(*t);
                    }
                    Type::ObjectRef(_) => {
                        vec.push(PrimitiveType::Hash);
                    }
                    Type::Object(name) => {
                        let st = structs.get(name).unwrap();
                        visit(structs, &mut vec, &st.fields);
                    }
                }
            }
        }

        visit(structs, &mut type_vec, fields);
    }
}
