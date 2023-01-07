use dioxus::prelude::*;
use dioxus_router::{use_route, use_router};

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
            .single_with_params([id]);
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
            text: content,
        }
        button {
            onclick: move |_| {
                db.execute("UPDATE cards SET content = ? WHERE card_id = ?")
                    .single_with_params((content.current().as_ref(), id));
                router.pop_route();
            },
            "Save"
        }
    })
}
