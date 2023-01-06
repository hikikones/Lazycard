use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use config::Config;
use database::Database;

mod components;
mod hooks;
mod routes;

fn main() {
    dioxus::desktop::launch(|cx| {
        cx.use_hook(|_| {
            let cfg = Config::default();
            let db = Database::open("db.db").unwrap();
            cx.provide_context(Rc::new(RefCell::new(cfg)));
            cx.provide_context(Rc::new(db));
        });

        cx.render(rsx! {
            App {}
        })
    });
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
            main {
                Route { to: "/review", routes::Review {} }
                Route { to: "/add_card", routes::AddCard {} }
                Route { to: "/edit_card/:id", routes::EditCard {} }
                Route { to: "/cards", routes::Cards {} }
                Route { to: "/settings", routes::Settings {} }
                Redirect { from: "", to: "/review" }
            }
        }
    })
}
