use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

mod config;
mod routes;

fn main() {
    dioxus::desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.use_hook(|_| {
        let cfg = config::Config::default();
        let db: Option<database::Database> = None;
        cx.provide_context(Rc::new(RefCell::new(cfg)));
        cx.provide_context(Rc::new(RefCell::new(db)));
    });

    cx.render(rsx! {
        Router {
            nav {
                ul {
                    Link { to: "/review", li { "Review"  }}
                    Link { to: "/cards", li { "Cards"  }}
                    Link { to: "/add_card", li { "Add card"  }}
                    Link { to: "/edit_card/1", li { "Edit card"  }}
                    Link { to: "/settings", li { "Settings"  }}
                }
            }
            main {
                Route { to: "/review", routes::Review {} }
                Route { to: "/cards", routes::Cards {} }
                Route { to: "/add_card", routes::AddCard {} }
                Route { to: "/edit_card/:id", routes::EditCard {} }
                Route { to: "/settings", routes::Settings {} }
                Route { to: "/load", routes::Load {} }
                Redirect { from: "", to: "/load" }
            }
        }
    })
}
