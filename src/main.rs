use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use config::Config;
use database::Database;

mod app;
mod components;
mod hooks;
mod routes;

fn main() {
    dioxus_desktop::launch_cfg(
        |cx| {
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
                sir::AppStyle {}
                app::App { initial_route: initial_route }
            })
        },
        dioxus_desktop::Config::new().with_custom_head(format!(
            r#"
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css">
            <style>{}</style>
        "#,
            include_str!("css/main.css"),
        ))
    );
}
