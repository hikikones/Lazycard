use dioxus::prelude::*;
use dioxus_router::use_router;

use database::Card;

use crate::{components::MarkdownView, hooks::use_database};

#[allow(non_snake_case)]
pub fn Cards(cx: Scope) -> Element {
    let db = use_database(&cx);
    let router = use_router(&cx);
    let cards = use_state(&cx, || {
        db.borrow()
            .fetch_all::<Card>("SELECT id, content FROM cards", [], |row| {
                Ok(Card {
                    id: row.get(0).unwrap(),
                    content: row.get(1).unwrap(),
                })
            })
            .unwrap()
    });

    cx.render(rsx! {
        h1 { "Cards" }
        cards.iter().map(|c| {
            rsx! {
                div {
                    MarkdownView {
                        key: "{c.id}",
                        text: "{c.content}",
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
