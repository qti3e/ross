use std::collections::HashMap;

use crate::ast;

#[derive(Debug)]
pub struct Lock<'a> {
    types: HashMap<u32, &'a Vec<ast::PrimitiveType>>,
}

impl<'a> Default for Lock<'a> {
    fn default() -> Self {
        Lock {
            types: HashMap::new(),
        }
    }
}

impl<'a> From<&'a ast::Mod> for Lock<'a> {
    fn from(module: &'a ast::Mod) -> Self {
        let mut lock = Lock::default();

        fn collect<'l>(mut lock: &mut Lock<'l>, module: &'l ast::Mod) {
            for (_, st) in &module.structs {
                lock.types.insert(st.id, &st.type_vec);
            }

            for (_, m) in &module.mods {
                collect(&mut lock, m);
            }
        }

        collect(&mut lock, module);

        lock
    }
}
