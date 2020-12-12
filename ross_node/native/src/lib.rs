use neon::prelude::*;
use ross_core::prelude::*;
use ross_core::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct JsContextOptions {
    editor_ttl: Option<Timestamp>,
    editors_cache_capacity: Option<usize>,
    max_number_of_editors: Option<usize>
}

declare_types! {
    pub class JsContext for ContextSync {
        init(mut cx) {
            let path = cx.argument::<JsString>(0)?;

            let options: Option<JsContextOptions> = match cx.argument_opt(1) {
                Some(arg) => Some(neon_serde::from_value(&mut cx, arg)?),
                _ => None
            };

            let mut opts = ContextOptionsBuilder::default()
                .path(path.value());

            let ctx = Context::new(&opts.build());
            Ok(ContextSync::new(ctx))
        }
    }
}

register_module!(mut m, {
    m.export_class::<JsContext>("Context")
});
