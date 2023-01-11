use std::path::Path;

use dioxus::prelude::*;
use dioxus_router::{use_route, use_router};

use database::*;

use crate::{components::MarkdownEditor, hooks::use_database};

#[allow(non_snake_case)]
pub fn AddCard(cx: Scope) -> Element {
    let db = use_database(&cx);
    let content = use_state(&cx, || String::new());

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Add Card" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        button {
            onclick: move |_| {
                db.borrow()
                    .execute_one(
                        "INSERT INTO cards (content) VALUES (?)",
                        [content.current()],
                    )
                    .unwrap();
                store_media(&html, &db.borrow());
                content.set(String::new());
            },
            "Add"
        }
    })
}

#[allow(non_snake_case)]
pub fn EditCard(cx: Scope) -> Element {
    let id = use_route(&cx)
        .segment("id")
        .unwrap()
        .parse::<SqliteId>()
        .unwrap();
    let db = use_database(&cx);
    let router = use_router(&cx);
    let content = use_state(&cx, || {
        db.borrow()
            .fetch_one::<String>("SELECT id, content FROM cards WHERE id = ?", [id], |row| {
                row.get(1)
            })
            .unwrap()
    });

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Edit Card ({id})" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        button {
            onclick: move |_| {
                db.borrow()
                    .execute_one(
                        "UPDATE cards SET content = ? WHERE id = ?",
                        (content.current().as_ref(), id),
                    )
                    .unwrap();
                store_media(&html, &db.borrow());
                router.pop_route();
            },
            "Save"
        }
    })
}

fn store_media(html: &str, db: &Database) {
    // const ASSET_REGEX: &str = const_format::concatcp!(r#"src\s*=\s*["|']"#, config::ASSETS_DIR, r#"/(.+?)["|']"#);
    const ASSET_REGEX: &str = const_format::concatcp!(config::ASSETS_DIR, r"/[0-9]*.[\w]*");

    let re = regex::Regex::new(ASSET_REGEX).unwrap();
    for caps in re.captures_iter(html) {
        let asset_path = Path::new(&caps[0]);

        let Some(file_stem) = asset_path.file_stem() else {
            return;
        };
        let Some(ext) = asset_path.extension() else {
            return;
        };
        let Ok(hash) = file_stem.to_string_lossy().parse::<Seahash>() else {
            return;
        };

        if db
            .fetch_one::<Seahash>(
                "SELECT seahash FROM media WHERE seahash = ?",
                [hash],
                |row| row.get(0),
            )
            .is_err()
        {
            let bytes = std::fs::read(asset_path).unwrap();
            db.execute_one(
                "INSERT INTO media (seahash, bytes, file_ext) VALUES (?, ?, ?)",
                params![hash, bytes, ext.to_string_lossy()],
            )
            .unwrap();
        }
    }
}
