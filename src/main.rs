use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use config::Config;
use database::Database;
use markdown::Markdown;

mod app;
mod components;
mod hooks;
mod routes;

fn main() {
    dioxus_desktop::launch_cfg(
        setup,
        dioxus_desktop::Config::new().with_custom_head(format!(
            r#"
        <!-- TODO -->
        "#
        )),
    );
}

fn setup(cx: Scope) -> Element {
    let cfg = &*cx.use_hook(|| {
        let cfg = Config::new();
        cx.provide_context(Rc::new(Markdown::default()));
        cx.provide_context(Rc::new(RefCell::new(cfg)))
    });

    if let Some(db_path) = cfg.borrow().database() {
        if !db_path.exists() {
            return cx.render(rsx! {
                h1 { "TODO: Database not found" }
                button {
                    onclick: move |_| {
                        let path = "db.db";
                        Database::open(path).unwrap();
                        cfg.borrow_mut().set_database(path.into());
                        cx.needs_update();
                    },
                    "New database"
                }
            });
        }

        if let Ok(db) = Database::open(db_path) {
            cx.use_hook(|| {
                cx.provide_context(Rc::new(db));
            });

            return cx.render(rsx! {
                app::App {}
            });
        }

        return cx.render(rsx! {
            h1 { "TODO: Could not open database" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    Database::open(path).unwrap();
                    cfg.borrow_mut().set_database(path.into());
                    cx.needs_update();
                },
                "New database"
            }
        });
    }

    cx.render(rsx! {
        h1 { "TODO: Greetings" }
        button {
            onclick: move |_| {
                let path = "db.db";
                Database::open(path).unwrap();
                cfg.borrow_mut().set_database(path.into());
                cx.needs_update();
            },
            "New database"
        }
    })
}
