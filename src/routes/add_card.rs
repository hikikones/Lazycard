use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn AddCard(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 { "Add card" }
    })
}
