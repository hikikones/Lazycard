use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router};

use crate::routes;

#[allow(non_snake_case)]
pub fn App(cx: Scope) -> Element {
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
