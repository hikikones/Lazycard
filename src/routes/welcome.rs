use dioxus::prelude::*;
use dioxus_router::prelude::*;

use config::use_config;
use database::use_database;

use crate::Route;

#[component]
pub fn Welcome() -> Element {
    let db = use_database();
    let cfg = use_config();
    let nav = use_navigator();

    rsx! {
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
    }
}
