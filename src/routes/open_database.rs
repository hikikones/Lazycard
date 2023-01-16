use std::path::Path;

use dioxus::prelude::*;
use dioxus_router::use_router;

use database::Seahash;

use crate::hooks::{use_config, use_database};

#[allow(non_snake_case)]
pub fn OpenDatabase(cx: Scope) -> Element {
    let db = use_database(&cx);
    let cfg = use_config(&cx);
    let router = use_router(&cx);

    let path = cfg.get_database_path().unwrap();

    if !path.exists() {
        return cx.render(rsx! {
            h1 { "Database not found" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.borrow_mut().open(path).unwrap();
                    cfg.set_database_path(path);
                    router.push_route("/review", None, None);
                },
                "New database"
            }
        });
    }

    let Ok(_) = db.borrow_mut().open(path) else {
        return cx.render(rsx! {
            h1 { "TODO: Could not open database" }
        });
    };

    let assets = db
        .borrow()
        .fetch_all::<(Seahash, String)>("SELECT seahash, extension FROM assets", [], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        })
        .unwrap();

    for (hash, ext) in assets {
        let asset_file = format!("{}/{}.{}", config::ASSETS_DIR, hash, ext);
        let asset_path = Path::new(&asset_file);
        if !asset_path.exists() {
            let bytes = db
                .borrow()
                .fetch_one::<Vec<u8>>(
                    "SELECT seahash, bytes FROM assets WHERE seahash = ?",
                    [hash],
                    |row| row.get(1),
                )
                .unwrap();
            std::fs::write(asset_path, bytes).unwrap();
        }
    }

    router.push_route("/review", None, None);

    cx.render(rsx! {
        h1 { "Success" }
    })
}
