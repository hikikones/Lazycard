use std::{borrow::Borrow, collections::HashSet};

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use database::{use_database, CardId};
use sqlite::params_from_iter;

use crate::{
    components::{MarkdownView, Tags},
    Route,
};

#[derive(Clone)]
struct Card {
    id: CardId,
    content: String,
}

#[component]
pub fn Cards() -> Element {
    let db = use_database();
    let nav = use_navigator();

    let mut cards = use_signal(|| Vec::<Card>::new());
    let mut selected_tags = use_signal(|| HashSet::new());
    let mut show_tagless = use_signal(|| false);

    use_effect(move || {
        let cards2 = if show_tagless() {
            db.fetch_all(
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
            let len = selected_tags.read().len();
            if len == 0 {
                db.fetch_all("SELECT id, content FROM cards", [], |row| {
                    Ok(Card {
                        id: row.get(0)?,
                        content: row.get(1)?,
                    })
                })
                .unwrap()
            } else {
                db.fetch_all(
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
                    params_from_iter(selected_tags.read().iter()),
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
        cards.set(cards2);
    });

    rsx! {
        h1 { "Cards" }

        h2 { "Tags" }
        button {
            onclick: move |_| {
                let current = show_tagless();
                show_tagless.set(!current);
            },
            "Tagless"
        }
        button {
            onclick: move |_| {
                if show_tagless() {
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

        {cards.read().iter().cloned().map(|c| {
            rsx! {
                div {
                    MarkdownView {
                        key: "{c.id}",
                        markdown: c.content,
                    }
                    button {
                        onclick: move |_| {
                            nav.push(Route::EditCard { id: c.id.raw() });
                        },
                        "Edit"
                    }
                }
            }
        })}
    }
}
