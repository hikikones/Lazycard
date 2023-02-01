use std::{collections::HashSet, path::Path};

use dioxus::prelude::*;
use dioxus_router::{use_route, use_router};

use database::*;
use once_cell::sync::Lazy;
use regex::Regex;
use sqlite::{params, SqliteId};

use crate::components::{MarkdownEditor, Tags};

#[allow(non_snake_case)]
pub fn AddCard(cx: Scope) -> Element {
    let db = use_database(&cx);
    let content = use_state(&cx, || String::new());
    let selected_tags = use_ref(&cx, || HashSet::new());

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Add Card" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        Tags {
            selected: selected_tags,
        }
        button {
            onclick: move |_| {
                add_card(&content.current(), &selected_tags.read(), &db);
                store_assets(&html, &db);
                content.set(String::new());
            },
            "Add"
        }
    })
}

fn add_card(content: &str, tags: &HashSet<TagId>, db: &Database) {
    db.execute_one("INSERT INTO cards (content) VALUES (?)", [content])
        .unwrap();

    let card_id = db.last_insert_rowid();

    tags.iter().copied().for_each(|tag_id| {
        db.execute_one(
            "INSERT INTO card_tag (card_id, tag_id) VALUES (?, ?)",
            (card_id, tag_id),
        )
        .unwrap();
    });
}

#[allow(non_snake_case)]
pub fn EditCard(cx: Scope) -> Element {
    let db = use_database(&cx);
    let route = use_route(&cx);
    let router = use_router(&cx);

    let card_id = *cx.use_hook(|| {
        let id = route.segment("id").unwrap().parse::<SqliteId>().unwrap();
        CardId::from_raw(id)
    });
    let content = use_state(&cx, || {
        db.fetch_one::<String>(
            "SELECT id, content FROM cards WHERE id = ?",
            [card_id],
            |row| row.get(1),
        )
        .unwrap()
    });
    let current_tags = use_ref(&cx, || {
        let mut tags = HashSet::<TagId>::new();
        db.fetch_with(
            "SELECT card_id, tag_id FROM card_tag WHERE card_id = ?",
            [card_id],
            |row| {
                tags.insert(row.get(1).unwrap());
            },
        )
        .unwrap();
        tags
    });
    let selected_tags = use_ref(&cx, || current_tags.read().clone());

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Edit Card ({card_id})" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        Tags {
            selected: selected_tags,
        }
        button {
            onclick: move |_| {
                edit_card(card_id, &content.current(), &current_tags.read(), &selected_tags.read(), &db);
                store_assets(&html, &db);
                router.pop_route();
            },
            "Save"
        }
    })
}

fn edit_card(
    card_id: CardId,
    content: &str,
    current_tags: &HashSet<TagId>,
    selected_tags: &HashSet<TagId>,
    db: &Database,
) {
    db.execute_one(
        "UPDATE cards SET content = ? WHERE id = ?",
        (content, card_id),
    )
    .unwrap();

    let to_add = selected_tags.difference(current_tags);
    let to_remove = current_tags.difference(selected_tags);

    to_add.copied().for_each(|tag_id| {
        db.execute_one(
            "INSERT INTO card_tag (card_id, tag_id) VALUES (?, ?)",
            (card_id, tag_id),
        )
        .unwrap();
    });

    to_remove.copied().for_each(|tag_id| {
        db.execute_one(
            "DELETE FROM card_tag WHERE card_id = ? AND tag_id = ?",
            (card_id, tag_id),
        )
        .unwrap();
    });
}

fn store_assets(html: &str, db: &Database) {
    // static ASSET_REGEX: Lazy<Regex> = Lazy::new(|| {
    //     Regex::new(&format!(
    //         r#"src\s*=\s*["|']{}/(.+?)["|']"#,
    //         config::ASSETS_DIR
    //     ))
    //     .unwrap()
    // });
    static ASSET_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(&format!(r"{}/[0-9]*.[\w]*", config::ASSETS_DIR)).unwrap());

    for caps in ASSET_REGEX.captures_iter(html) {
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
            // TODO: fetch_maybe
            .fetch_one::<Seahash>(
                "SELECT seahash FROM assets WHERE seahash = ?",
                [hash],
                |row| row.get(0),
            )
            .is_err()
        {
            let bytes = std::fs::read(asset_path).unwrap();
            db.execute_one(
                "INSERT INTO assets (seahash, bytes, extension) VALUES (?, ?, ?)",
                params![hash, bytes, ext.to_string_lossy()],
            )
            .unwrap();
        }
    }
}
