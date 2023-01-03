use dioxus::prelude::*;

use database::Card;

use crate::hooks::use_database;

#[allow(non_snake_case)]
pub fn Review(cx: Scope) -> Element {
    let db = use_database(&cx);

    let cards = db.fetch_all::<Card>("SELECT * FROM cards", []);
    for card in cards {
        dbg!(card.id);
    }

    cx.render(rsx! {
        h1 { "Review" }
        button {
            onclick: move |_| {
                let router = use_router(&cx);
                router.push_route("/load", None, None);
            },
            "Load"
        }
    })
}
