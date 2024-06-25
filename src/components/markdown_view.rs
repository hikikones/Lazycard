use dioxus::prelude::*;

#[component]
pub fn MarkdownView(markdown: String) -> Element {
    let html = markdown::to_html(markdown.as_str());

    rsx! {
        div {
            dangerous_inner_html: "{html}",
        }
    }
}
