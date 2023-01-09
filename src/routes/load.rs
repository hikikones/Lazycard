use std::path::Path;

use dioxus::prelude::*;
use dioxus_router::use_router;

use database::Seahash;

use crate::hooks::use_database;

#[allow(non_snake_case)]
pub fn Load(cx: Scope) -> Element {
    let db = use_database(&cx).clone();
    let router = use_router(&cx).clone();

    use_effect(&cx, (), |_| async move {
        let assets = db
            .borrow()
            .fetch::<(Seahash, String)>("SELECT seahash, file_ext FROM media")
            .all();

        for (hash, ext) in assets {
            let asset_file = format!("{}/{}.{}", config::ASSETS_DIR, hash.raw(), ext);
            let asset_path = Path::new(&asset_file);
            if !asset_path.exists() {
                let (_, bytes) = db
                    .borrow()
                    .fetch::<(Seahash, Vec<u8>)>(
                        "SELECT seahash, bytes FROM media WHERE seahash = ?",
                    )
                    .single_with_params([hash]);
                std::fs::write(asset_path, bytes).unwrap();
            }
        }

        router.push_route("/review", None, None);
    });

    cx.render(rsx! {
        h1 { "Load" }
    })
}
