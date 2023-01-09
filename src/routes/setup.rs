use dioxus::prelude::*;
use dioxus_router::use_router;

use crate::hooks::{use_config, use_database};

#[allow(non_snake_case)]
pub fn Setup(cx: Scope) -> Element {
    let cfg = use_config(&cx);
    let db = use_database(&cx);
    let router = use_router(&cx);

    let Some(db_path) = cfg.borrow().database().cloned() else {
        return cx.render(rsx! {
            h1 { "TODO: Greetings" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.borrow_mut().open(path).unwrap();
                    cfg.borrow_mut().set_database(path.into());
                    cx.needs_update();
                },
                "New database"
            }
        });
    };

    if !db_path.exists() {
        return cx.render(rsx! {
            h1 { "TODO: Database not found" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.borrow_mut().open(path).unwrap();
                    cfg.borrow_mut().set_database(path.into());
                    cx.needs_update();
                },
                "New database"
            }
        });
    }

    let Ok(_) = db.borrow_mut().open(db_path) else {
        return cx.render(rsx! {
            h1 { "TODO: Could not open database" }
        });
    };

    router.push_route("/load", None, None);

    None
}
