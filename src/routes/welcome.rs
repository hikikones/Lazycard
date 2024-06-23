use dioxus::prelude::*;
use dioxus_router::prelude::*;

use config::use_config;
use database::use_database;

use crate::Route;

#[allow(non_snake_case)]
pub fn Welcome(cx: Scope) -> Element {
    let db = use_database(&cx);
    let cfg = use_config(&cx);
    let nav = use_navigator(cx);

    cx.render(rsx! {
        h1 { "Welcome" }
        button {
            onclick: move |_| {
                let path = "db.db";
                db.open(path).unwrap();
                cfg.set_database_path(path);
                nav.push(Route::Review {  });
            },
            "New database"
        }
    })
}
