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
                db.borrow().execute("INSERT INTO cards (content) VALUES (?)")
                    .single_with_params([content.current()]);
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
        let (_, content) = db
            .borrow()
            .fetch::<(SqliteId, String)>("SELECT * FROM cards WHERE card_id = ?")
            .single_with_params([id]);
        content
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
                db.borrow().execute("UPDATE cards SET content = ? WHERE card_id = ?")
                    .single_with_params((content.current().as_ref(), id));
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
        let Ok(hash) = file_stem.to_string_lossy().parse::<u64>() else {
            return
        };

        let seahash = Seahash::from_raw(hash);

        if db
            .fetch::<Seahash>("SELECT seahash FROM media WHERE seahash = ?")
            .get_single_with_params([seahash])
            .is_none()
        {
            let bytes = std::fs::read(asset_path).unwrap();
            db.execute("INSERT INTO media (seahash, bytes, file_ext) VALUES (?, ?, ?)")
                .single_with_params(params![seahash, bytes, ext.to_string_lossy()]);
        }
    }
}
