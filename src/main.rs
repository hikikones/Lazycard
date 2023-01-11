use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router};

use config::Config;
use database::Database;

mod components;
mod hooks;
mod routes;

fn main() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::new().with_custom_head(format!(
            r#"
        <!-- TODO -->
        "#
        )),
    );
}

pub fn app(cx: Scope) -> Element {
    cx.use_hook(|| {
        cx.provide_context(Rc::new(RefCell::new(Config::new())));
        cx.provide_context(Rc::new(RefCell::new(Database::new())));
    });

    let show_nav = use_state(&cx, || false);

    cx.render(rsx! {
        Router {
            show_nav.current().then(|| rsx! {
                nav {
                    ul {
                        Link { to: "/review", li { "Review" }}
                        Link { to: "/add_card", li { "Add Card" }}
                        Link { to: "/cards", li { "Cards" }}
                        Link { to: "/settings", li { "Settings" }}
                    }
                }
            })
            Route { to: "/setup", main { routes::Setup { show_nav: show_nav } } }
            Route { to: "/review", main { routes::Review {} } }
            Route { to: "/add_card", main { routes::AddCard {} } }
            Route { to: "/edit_card/:id", main { routes::EditCard {} } }
            Route { to: "/cards", main { routes::Cards {} } }
            Route { to: "/settings", main { routes::Settings {} } }
            Redirect { from: "", to: "/setup" }
        }
    })
}
