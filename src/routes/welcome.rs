use dioxus::prelude::*;
use dioxus_router::use_router;

use crate::hooks::{use_config, use_database};

#[allow(non_snake_case)]
pub fn Welcome(cx: Scope) -> Element {
    let db = use_database(&cx);
    let cfg = use_config(&cx);
    let router = use_router(&cx);

    cx.render(rsx! {
        h1 { "Welcome" }
        button {
            onclick: move |_| {
                let path = "db.db";
                db.borrow_mut().open(path).unwrap();
                cfg.set_database_path(path);
                router.push_route("/review", None, None);
            },
            "New database"
        }
    })
}