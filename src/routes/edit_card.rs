use dioxus::prelude::*;

use database::*;

use crate::{
    components::{MarkdownEditor, MarkdownView},
    hooks::use_database,
};

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
            .fetch::<(SqliteId, String)>("SELECT * FROM cards WHERE card_id = ?")
            .args([id])
            .single();
        content
    });

    cx.render(rsx! {
        h1 { "Edit Card ({id})" }
        MarkdownEditor {
            value: "{content}",
            oninput: |text| {
                content.set(text);
            }
        }
        MarkdownView {
            markdown: content,
        }
        button {
            onclick: move |_| {
                db.execute("UPDATE cards SET content = ? WHERE card_id = ?")
                    .args(params![content.current(), id])
                    .single();
                router.pop_route();
            },
            "Save"
        }
    })
}
