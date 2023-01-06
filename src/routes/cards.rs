use dioxus::prelude::*;

use database::Card;

use crate::{components::MarkdownView, hooks::use_database};

#[allow(non_snake_case)]
pub fn Cards(cx: Scope) -> Element {
    let db = use_database(&cx);
    let router = use_router(&cx);
    let cards = use_state(&cx, || db.fetch::<Card>("SELECT * FROM cards").all());

    cx.render(rsx! {
        h1 { "Cards" }
        cards.iter().map(|c| {
            rsx! {
                div {
                    MarkdownView {
                        key: "{c.id}",
                        markdown: "{c.content}",
                    }
                    button {
                        onclick: |_| {
                            router.push_route(&format!("/edit_card/{}", c.id), None, None);
                        },
                        "Edit"
                    }
                }
            }
        })
    })
}
