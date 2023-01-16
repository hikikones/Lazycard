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
    let initial_route = *cx.use_hook(|| {
        cx.provide_context(Rc::new(RefCell::new(Database::new())));
        let cfg = cx.provide_context(Rc::new(Config::new()));
        if cfg.get_database_path().is_some() {
            "/open_database"
        } else {
            "/welcome"
        }
    });

    cx.render(rsx! {
        Router {
            Route { to: "/welcome", routes::Welcome {} }
            Route { to: "/open_database", routes::OpenDatabase {} }
            Route { to: "/review", Main { routes::Review {} } }
            Route { to: "/add_card", Main { routes::AddCard {} } }
            Route { to: "/edit_card/:id", Main { routes::EditCard {} } }
            Route { to: "/cards", Main { routes::Cards {} } }
            Route { to: "/settings", Main { routes::Settings {} } }
            Redirect { from: "", to: initial_route }
        }
    })
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Main<'a>(cx: Scope, children: Element<'a>) -> Element {
    cx.render(rsx! {
        Navigation {}
        main {
            children
        }
    })
}

#[allow(non_snake_case)]
pub fn Navigation<'a>(cx: Scope) -> Element {
    cx.render(rsx! {
        nav {
            ul {
                Link { to: "/review", li { "Review" }}
                Link { to: "/add_card", li { "Add Card" }}
                Link { to: "/cards", li { "Cards" }}
                Link { to: "/settings", li { "Settings" }}
            }
        }
    })
}
