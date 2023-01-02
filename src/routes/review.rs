use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn Review(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 { "Review" }
    })
}
