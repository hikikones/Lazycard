use dioxus::prelude::*;
use sir::{css, global_css};

mod components;
mod routes;

use components::{Icon, IconName, IconSize};

fn main() {
    let custom_head = format!(
        r#"
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css">
        <style>{}</style>
    "#,
        include_str!("css/main.css"),
    );

    LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_menu(None)
                .with_custom_head(custom_head),
        )
        .launch(app);
}

use routes::*;
use sqlite::SqliteId;

#[derive(Routable, PartialEq, Clone)]
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

pub fn app() -> Element {
    database::provide_database();
    config::provide_config();

    rsx! {
        sir::AppStyle {}
        Router::<Route> {}
    }
}

#[component]
fn Redirect() -> Element {
    let cfg = config::use_config();
    let nav = use_navigator();

    if cfg.get_database_path().is_some() {
        nav.push(Route::OpenDatabase {});
    } else {
        nav.push(Route::Welcome {});
    }

    None
}

#[allow(non_snake_case)]
fn Main() -> Element {
    global_css!(
        "
        #main {
            display: flex;
        }       
    "
    );

    rsx! {
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
    }
}

#[component]
fn NavLink(props: NavLinkProps) -> Element {
    rsx! {
        Link {
            to: props.to,
            active_class: "active",
            class: css!("
                display: flex;
                padding: 10px;
                color: var(--primary-variant-color);
                &:hover, &.active {
                    color: var(--primary-text-color);
                }
            "),
            Icon {
                name: props.icon,
                size: IconSize::Custom(28),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NavLinkProps {
    to: &'static str,
    icon: IconName,
}
