use dioxus::prelude::*;
use dioxus_router::prelude::*;
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

use routes::*;
use sqlite::SqliteId;

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Redirect {},
    #[route("/welcome")]
    Welcome {},
    #[route("/open_database")]
    OpenDatabase {},

    #[layout(Main)]
    #[route("/review")]
    Review {},
    #[route("/add_card")]
    AddCard {},
    #[route("/edit_card/:id")]
    EditCard { id: SqliteId },
    #[route("/cards")]
    Cards {},
    #[route("/settings")]
    Settings {},
}

pub fn app(cx: Scope) -> Element {
    database::provide_database(cx);
    config::provide_config(cx);

    cx.render(rsx! {
        sir::AppStyle {}
        Router::<Route> {}
    })
}

#[component]
fn Redirect(cx: Scope) -> Element {
    let cfg = config::use_config(cx);
    let nav = use_navigator(cx);

    if cfg.get_database_path().is_some() {
        nav.push(Route::OpenDatabase {});
    } else {
        nav.push(Route::Welcome {});
    }

    None
}

#[allow(non_snake_case)]
#[inline_props]
fn Main(cx: Scope) -> Element {
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

            Outlet::<Route> {}
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
