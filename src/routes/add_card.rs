use dioxus::prelude::*;

use crate::{
    components::{MarkdownEditor, MarkdownView},
    hooks::use_database,
};

#[allow(non_snake_case)]
pub fn AddCard(cx: Scope) -> Element {
    let content = use_state(&cx, || String::new());
    let db = use_database(&cx);

    cx.render(rsx! {
        h1 { "Add Card" }
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
                db.execute_one(
                    "INSERT INTO cards (content) VALUES (?)",
                    [content.current()],
                );
                content.set(String::new());
            },
            "Add"
        }
    })
}
