use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use database::*;

use crate::config::Config;

#[allow(non_snake_case)]
pub fn Load(cx: Scope) -> Element {
    let router = use_router(&cx);
    let has_loaded = use_state(&cx, || false);

    let cfg = cx.use_hook(|_| {
        let cfg = Config::default();
        cx.provide_root_context(Rc::new(RefCell::new(cfg)))
    });

    let db = cx.use_hook(|_| {
        let path = cfg.borrow().database.clone().unwrap_or("db.db".into());
        let db = Database::open(path).unwrap();
        cx.provide_root_context(Rc::new(db))
    });

    has_loaded.then(|| {
        router.push_route("/review", None, None);
    });

    use_effect(&cx, has_loaded, |has_loaded| async move {
        has_loaded.set(true);
    });

    cx.render(rsx! {
        h1 { "Load" }
    })
}
