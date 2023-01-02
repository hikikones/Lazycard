use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn Cards(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 { "All Cards" }
    })
}
