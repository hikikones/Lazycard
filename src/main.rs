use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router};
use sir::{css, global_css};

mod components;
mod routes;

use components::{Icon, IconName, IconSize};

fn main() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::new().with_custom_head(format!(
            r#"
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css">
            <style>{}</style>
        "#,
            include_str!("css/main.css"),
        ))
    );
}

pub fn app(cx: Scope) -> Element {
    database::provide_database(cx);
    let cfg = config::provide_config(cx);

    let initial_route = if cfg.get_database_path().is_some() {
        "/open_database"
    } else {
        "/welcome"
    };

    cx.render(rsx! {
        sir::AppStyle {}

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
fn Main<'a>(cx: Scope, children: Element<'a>) -> Element {
    global_css!(
        "
        #main {
            display: flex;
        }       
    "
    );

    cx.render(rsx! {
        nav {
            class: css!("
                position: sticky;
                top: 0;
                height: 100vh;
                min-width: 48px;
                max-width: 48px;
                background-color: var(--primary-color);
            "),

            div {
                class: css!("
                    display: flex;
                    height: inherit;
                    flex-direction: column;
                    justify-content: space-between;
                "),

                div {
                    NavLink { to: "/review", icon: IconName::Drafts }
                    NavLink { to: "/cards", icon: IconName::Layers }
                    NavLink { to: "/add_card", icon: IconName::AddCircle }
                }

                div {
                    NavLink { to: "/settings", icon: IconName::Settings }
                }
            }
        }

        main {
            class: css!("
                flex-grow: 1;
                padding: 1rem;
            "),

            children
        }
    })
}

#[allow(non_snake_case)]
fn NavLink<'a>(cx: Scope<'a, NavLinkProps<'a>>) -> Element {
    cx.render(rsx! {
        Link {
            to: cx.props.to,
            class: css!("
                display: flex;
                padding: 10px;
                color: var(--primary-variant-color);
                &:hover, &.active {
                    color: var(--primary-text-color);
                }
            "),
            Icon {
                name: cx.props.icon,
                size: IconSize::Custom(28),
            }
        }
    })
}

#[derive(Props)]
struct NavLinkProps<'a> {
    to: &'a str,
    icon: IconName,
}
