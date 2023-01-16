use std::ops::Sub;

use dioxus::prelude::*;

use database::{CardId, Database};

use crate::{components::MarkdownView, hooks::use_database};

struct Card {
    id: CardId,
    content: String,
}

#[allow(non_snake_case)]
pub fn Review(cx: Scope) -> Element {
    let db = use_database(&cx);
    let total = *cx.use_hook(|| get_due_count(&db.borrow()));

    if total == 0 {
        return cx.render(rsx! {
            h1 { "No cards to review" }
        });
    }

    let count = use_state(&cx, || 0);

    if **count == total {
        return cx.render(rsx! {
            h1 { "Good job" }
        });
    }

    let card = use_state(&cx, || get_due_card(&db.borrow()));

    cx.render(rsx! {
        h1 { "Review" }

        span { "{count} / {total}" }
        br {}

        MarkdownView {
            text: &card.content
        }

        div {
            button {
                onclick: move |_| {
                    update_card_review(card, true, &db.borrow());
                    count.with_mut(|c| {
                        *c += 1;
                        if *c < total {
                            card.set(get_due_card(&db.borrow()));
                        }
                    });
                },
                "Yes"
            }
            button {
                onclick: move |_| {
                    update_card_review(card, false, &db.borrow());
                    count.with_mut(|c| {
                        *c += 1;
                        if *c < total {
                            card.set(get_due_card(&db.borrow()));
                        }
                    });
                },
                "No"
            }
            button {
                onclick: move |_| {
                    if **count < total - 1 {
                        card.set(get_due_card_except(card.id, &db.borrow()));
                    }
                },
                "Skip"
            }
        }
    })
}

fn get_due_count(db: &Database) -> u32 {
    db.fetch_one(
        "SELECT COUNT(id) FROM cards WHERE review_date <= (date('now'))",
        [],
        |row| row.get(0),
    )
    .unwrap()
}

fn get_due_card(db: &Database) -> Card {
    db.fetch_one(
        "
            SELECT id, content, review_date FROM cards \
            WHERE review_date <= (date('now')) \
            ORDER BY RANDOM() \
            LIMIT 1
            ",
        [],
        |row| {
            Ok(Card {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        },
    )
    .unwrap()
}

fn get_due_card_except(id: CardId, db: &Database) -> Card {
    db.fetch_one(
        "
            SELECT id, content, review_date FROM cards \
            WHERE review_date <= (date('now')) \
            AND id != ? \
            ORDER BY RANDOM() \
            LIMIT 1
            ",
        [id],
        |row| {
            Ok(Card {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        },
    )
    .unwrap()
}

fn update_card_review(card: &Card, success: bool, db: &Database) {
    db.execute_one(
        "INSERT INTO reviews (success, card_id) VALUES (?, ?)",
        (success, card.id),
    )
    .unwrap();

    let successes = db
        .fetch_all::<bool>(
            "SELECT success, card_id FROM reviews WHERE card_id = ?",
            [card.id],
            |row| row.get(0),
        )
        .unwrap();
    let exp = successes
        .into_iter()
        .map(|s| if s { 1 } else { -1 })
        .sum::<i32>()
        .sub(1)
        .max(0) as u32;
    let days = i64::pow(2, exp);

    let next_review = chrono::Utc::now() + chrono::Duration::days(days);

    db.execute_one(
        "UPDATE cards SET review_date = ? WHERE id = ?",
        (next_review.date_naive(), card.id),
    )
    .unwrap();
}
