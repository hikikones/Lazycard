use std::path::Path;

use dioxus::prelude::*;
use dioxus_router::use_router;

use database::Seahash;

use crate::hooks::{use_config, use_database};

#[allow(non_snake_case)]
pub fn Setup(cx: Scope) -> Element {
    let cfg = use_config(&cx);
    let db = use_database(&cx);
    let router = use_router(&cx);

    let Some(db_path) = cfg.borrow().database().cloned() else {
        return cx.render(rsx! {
            h1 { "TODO: Welcome" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.borrow_mut().open(path).unwrap();
                    cfg.borrow_mut().set_database(path.into());
                    cx.needs_update();
                },
                "New database"
            }
        });
    };

    if !db_path.exists() {
        return cx.render(rsx! {
            h1 { "TODO: Database not found" }
            button {
                onclick: move |_| {
                    let path = "db.db";
                    db.borrow_mut().open(path).unwrap();
                    cfg.borrow_mut().set_database(path.into());
                    cx.needs_update();
                },
                "New database"
            }
        });
    }

    let Ok(_) = db.borrow_mut().open(db_path) else {
        return cx.render(rsx! {
            h1 { "TODO: Could not open database" }
        });
    };

    let assets = db
        .borrow()
        .fetch_all::<(Seahash, String)>("SELECT seahash, extension FROM media", [], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        })
        .unwrap();

    for (hash, ext) in assets {
        let asset_file = format!("{}/{}.{}", config::ASSETS_DIR, hash.raw(), ext);
        let asset_path = Path::new(&asset_file);
        if !asset_path.exists() {
            let bytes = db
                .borrow()
                .fetch_one::<Vec<u8>>(
                    "SELECT seahash, bytes FROM media WHERE seahash = ?",
                    [hash],
                    |row| row.get(1),
                )
                .unwrap();
            std::fs::write(asset_path, bytes).unwrap();
        }
    }

    let router_clone = router.clone();
    use_effect(&cx, (), |_| async move {
        router_clone.push_route("/review", None, None);
    });

    cx.render(rsx! {
        h1 { "Success" }
    })
}
