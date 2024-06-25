use std::ops::Sub;

use dioxus::prelude::*;
use sir::css;

use database::{use_database, CardId, Database};

use crate::components::{Button, ButtonBorder, ButtonPadding, Icon, IconName, IconSize};

#[component]
pub fn Review() -> Element {
    let db = use_database();
    let total_cards = use_hook(|| get_due_count(&db));

    if total_cards == 0 {
        return rsx! {
            h1 { "No cards to review" }
        };
    }

    let mut review_count = use_signal(|| 0);

    if review_count() == total_cards {
        return rsx! {
            h1 { "Good job" }
        };
    }

    let mut card_id = use_signal(|| CardId::ZERO);
    let mut card_content = use_signal(|| Vec::new());
    let mut show_count = use_signal(|| 1);

    let db_clone = db.clone();
    use_effect(move || {
        let next_id = get_due_card_id(&db_clone);
        card_id.set(next_id);
    });

    let db_clone = db.clone();
    use_effect(move || {
        if card_id().raw() == 0 {
            return;
        }

        let content = get_card_content(card_id(), &db_clone);
        let split = content
            .split("\n\n---\n\n")
            .map(|s| s.to_string())
            .collect();
        card_content.set(split);
        show_count.set(1);
    });

    let card_render = card_content
        .iter()
        .take(show_count())
        .enumerate()
        .map(|(i, s)| {
            let last = i == show_count() - 1;
            rsx! {
                div {
                    padding: "0 1rem",
                    dangerous_inner_html: format_args!("{}", markdown::to_html(s.as_str())),
                },
                {(!last).then(|| rsx!(hr {}))}
            }
        });

    let db2 = db.clone();
    let db3 = db.clone();
    let button_render = match show_count() < card_content.len() {
        true => rsx! {
            ReviewButton {
                icon: IconName::LockOpen,
                onclick: move |_| {
                    show_count += 1;
                }
            }
        },
        false => rsx! {
            ReviewButton {
                icon: IconName::Done,
                onclick: move |_| {
                    update_card_review(card_id(), true, &db2);
                    *review_count.write() += 1;
                }
            }
            ReviewButton {
                icon: IconName::Close,
                onclick: move |_| {
                    update_card_review(card_id(), false, &db3);
                    *review_count.write() += 1;
                }
            }
        },
    };

    let db4 = db.clone();

    rsx! {
        div {
            class: css!("
                display: flex;
                height: 100%;
                flex-direction: column;
                justify-content: space-between;
                align-items: center;
            "),

            div {
                span {
                    opacity: "0.5",
                    "{review_count} / {total_cards}"
                }
            }

            div {
                class: css!("
                    max-width: 75ch;
                    padding: 1rem;
                    margin: 1rem;
                    border: none;
                    box-shadow: 0 0.25rem 1rem var(--shadow-color);
                    background-color: var(--surface-color);
                    color: var(--surface-text-color);
                "),
                {card_render}
            }

            div {
                class: css!("
                    & > * {
                        margin: 0 0.5rem;
                    }
                "),

                {button_render},

                ReviewButton {
                    icon: IconName::DoubleArrow,
                    onclick: move |_| {
                        let next_id = get_due_card_id_except(card_id(), &db4);
                        card_id.set(next_id);
                    }
                }
            }
        }
    }
}

#[component]
fn ReviewButton(onclick: EventHandler<Event<MouseData>>, icon: IconName) -> Element {
    rsx! {
        Button {
            border: ButtonBorder::Circle,
            padding: ButtonPadding::Custom("1rem"),
            onclick: move |event| {
                onclick.call(event);
            },
            Icon {
                name: icon,
                size: IconSize::Large,
            }
        }
    }
}

fn get_due_count(db: &Database) -> u32 {
    db.fetch_one(
        "SELECT COUNT(id) FROM cards WHERE review_date <= (date('now'))",
        [],
        |row| row.get(0),
    )
    .unwrap()
}

fn get_due_card_id(db: &Database) -> CardId {
    get_due_card_id_except(CardId::ZERO, db)
}

fn get_due_card_id_except(id: CardId, db: &Database) -> CardId {
    db.fetch_one(
        "
            SELECT id, review_date FROM cards \
            WHERE review_date <= (date('now')) AND id != ? \
            ORDER BY RANDOM() \
            LIMIT 1
            ",
        [id],
        |row| Ok(row.get(0)?),
    )
    .unwrap()
}

fn get_card_content(id: CardId, db: &Database) -> String {
    db.fetch_one("SELECT id, content FROM cards WHERE id = ?", [id], |row| {
        Ok(row.get(1)?)
    })
    .unwrap()
}

fn update_card_review(id: CardId, success: bool, db: &Database) {
    db.execute_one(
        "INSERT INTO reviews (success, card_id) VALUES (?, ?)",
        (success, id),
    )
    .unwrap();

    let successes = db
        .fetch_all::<bool>(
            "SELECT success, card_id FROM reviews WHERE card_id = ?",
            [id],
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
        (next_review.date_naive(), id),
    )
    .unwrap();
}
