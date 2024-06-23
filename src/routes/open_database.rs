use std::path::Path;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use config::use_config;
use database::{use_database, Seahash};

use crate::Route;

#[allow(non_snake_case)]
pub fn OpenDatabase(cx: Scope) -> Element {
    let db = use_database(&cx);
    let cfg = use_config(&cx);
    let nav = use_navigator(cx);

    let path = cfg.get_database_path().unwrap();

    if !path.exists() {
        return cx.render(rsx! {
            h1 { "Database not found" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.open(path).unwrap();
                    cfg.set_database_path(path);
                    nav.push(Route::Review {  });
                },
                "New database"
            }
        });
    }

    let Ok(_) = db.open(path) else {
        return cx.render(rsx! {
            h1 { "TODO: Could not open database" }
        });
    };

    let assets = db
        .fetch_all::<(Seahash, String)>("SELECT seahash, extension FROM assets", [], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        })
        .unwrap();

    for (hash, ext) in assets {
        let asset_file = format!("{}/{}.{}", config::ASSETS_DIR, hash, ext);
        let asset_path = Path::new(&asset_file);
        if !asset_path.exists() {
            let bytes = db
                .fetch_one::<Vec<u8>>(
                    "SELECT seahash, bytes FROM assets WHERE seahash = ?",
                    [hash],
                    |row| row.get(1),
                )
                .unwrap();
            std::fs::write(asset_path, bytes).unwrap();
        }
    }

    nav.push(Route::Review {});

    cx.render(rsx! {
        h1 { "Success" }
    })
}
