use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router};
use sir::css;

use crate::{
    components::{Icon, IconName},
    routes,
};

#[allow(non_snake_case)]
#[inline_props]
pub fn App<'a>(cx: Scope, initial_route: &'a str) -> Element {
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
fn Main<'a>(cx: Scope, children: Element<'a>) -> Element {
    cx.render(rsx! {
        div {
            display: "flex",

            nav {
                class: css!("
                    position: sticky;
                    top: 0;
                    height: 100vh;
                    min-width: 48px;
                    max-width: 48px;
                    background: var(--primary-color);
                "),

                div {
                    class: css!("
                        display: flex;
                        height: inherit;
                        flex-direction: column;
                        justify-content: space-between;
                    "),

                    div {
                        NavLink { to: "/review", icon: IconName::DraftsFill }
                        NavLink { to: "/cards", icon: IconName::LayersFill }
                        NavLink { to: "/add_card", icon: IconName::AddCircleFill }
                    }

                    div {
                        NavLink { to: "/settings", icon: IconName::SettingsFill }
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
                &.active svg { fill: var(--primary-text-color); }
            "),
            Icon {
                name: cx.props.icon,
                class: css!("
                    padding: 10px;
                    &:hover { fill: var(--primary-text-color); }
                "),
                size: 28,
                fill: "var(--primary-variant-color)",
            }
        }
    })
}

#[derive(Props)]
struct NavLinkProps<'a> {
    to: &'a str,
    icon: IconName,
}
