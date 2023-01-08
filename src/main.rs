use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router};

use config::Config;
use database::Database;
use markdown::Markdown;

mod components;
mod hooks;
mod routes;

fn main() {
    dioxus_desktop::launch_cfg(
        init,
        dioxus_desktop::Config::new().with_custom_head(format!(
            r#"
        <!-- TODO -->
        "#
        )),
    );
}

fn init(cx: Scope) -> Element {
    let cfg = &*cx.use_hook(|| {
        let cfg = Config::default();
        cx.provide_context(Rc::new(Markdown::default()));
        cx.provide_context(Rc::new(RefCell::new(cfg)))
    });

    if let Some(db_path) = &cfg.borrow().database {
        if let Ok(db) = Database::open(db_path) {
            cx.use_hook(|| {
                cx.provide_context(Rc::new(db));
            });

            return cx.render(rsx! {
                App {}
            });
        } else {
            let db_path = db_path.clone();
            return cx.render(rsx! {
                h1 { "TODO: Database not found" }
                button {
                    onclick: move |_| {
                        Database::open(&db_path).unwrap();
                        cx.needs_update();
                    },
                    "New database"
                }
            });
        }
    }

    cx.render(rsx! {
        h1 { "TODO: Welcome" }
        button {
            onclick: move |_| {
                let path = "db.db";
                Database::open(path).unwrap();
                cfg.borrow_mut().database = Some(path.into());
                cx.needs_update();
            },
            "New database"
        }
    })
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            nav {
                ul {
                    Link { to: "/review", li { "Review" }}
                    Link { to: "/add_card", li { "Add Card" }}
                    Link { to: "/cards", li { "Cards" }}
                    Link { to: "/settings", li { "Settings" }}
                }
            }
            Route { to: "/review", main { routes::Review {} } }
            Route { to: "/add_card", main { routes::AddCard {} } }
            Route { to: "/edit_card/:id", main { routes::EditCard {} } }
            Route { to: "/cards", main { routes::Cards {} } }
            Route { to: "/settings", main { routes::Settings {} } }
            Redirect { from: "", to: "/review" }
        }
    })
}
