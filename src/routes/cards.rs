use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use database::{use_database, CardId};
use sqlite::params_from_iter;

use crate::{
    components::{MarkdownView, Tags},
    Route,
};

struct Card {
    id: CardId,
    content: String,
}

#[allow(non_snake_case)]
pub fn Cards(cx: Scope) -> Element {
    let db = use_database(&cx);
    let nav = use_navigator(cx);

    let cards = use_state(&cx, || Vec::new());
    let selected_tags = use_ref(&cx, || HashSet::new());
    let show_tagless = use_state(&cx, || false);

    let db_clone = db.clone();
    let cards_clone = cards.clone();

    use_effect(
        &cx,
        (selected_tags, show_tagless),
        |(selected, tagless)| async move {
            let cards = if *tagless.current() {
                db_clone
                    .fetch_all(
                        "
                        SELECT * FROM cards c WHERE NOT EXISTS ( \
                            SELECT ct.card_id FROM card_tag ct \
                            WHERE c.id = ct.card_id \
                        )",
                        [],
                        |row| {
                            Ok(Card {
                                id: row.get(0)?,
                                content: row.get(1)?,
                            })
                        },
                    )
                    .unwrap()
            } else {
                let len = selected.read().len();
                if len == 0 {
                    db_clone
                        .fetch_all("SELECT id, content FROM cards", [], |row| {
                            Ok(Card {
                                id: row.get(0)?,
                                content: row.get(1)?,
                            })
                        })
                        .unwrap()
                } else {
                    db_clone
                        .fetch_all(
                            &format!(
                                "
                                SELECT id, content FROM cards c \
                                JOIN card_tag ct ON ct.card_id = c.id WHERE tag_id IN ({}) \
                                GROUP BY ct.card_id HAVING Count(*) = {}
                                ",
                                (0..len)
                                    .into_iter()
                                    .map(|_| "?")
                                    .collect::<Box<[_]>>()
                                    .join(","),
                                len
                            ),
                            params_from_iter(selected.read().iter()),
                            |row| {
                                Ok(Card {
                                    id: row.get(0)?,
                                    content: row.get(1)?,
                                })
                            },
                        )
                        .unwrap()
                }
            };
            cards_clone.set(cards);
        },
    );

    cx.render(rsx! {
        h1 { "Cards" }

        h2 { "Tags" }
        button {
            onclick: |_| {
                show_tagless.set(!*show_tagless.current());
            },
            "Tagless"
        }
        button {
            onclick: |_| {
                if **show_tagless {
                    show_tagless.set(false);
                }
                if !selected_tags.read().is_empty() {
                    selected_tags.write().clear();
                }
            },
            "Reset"
        }
        br {}
        Tags {
            selected: selected_tags,
        }

        cards.iter().map(|c| rsx! {
            div {
                MarkdownView {
                    key: "{c.id}",
                    text: "{c.content}",
                }
                button {
                    onclick: |_| {
                        nav.push(Route::EditCard { id: c.id.raw() });
                    },
                    "Edit"
                }
            }
        })
    })
}
