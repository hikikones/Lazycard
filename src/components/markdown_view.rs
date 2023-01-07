use dioxus::prelude::*;

use crate::hooks::use_markdown;

#[allow(non_snake_case)]
pub fn MarkdownView<'a>(cx: Scope<'a, MarkdownViewProps<'a>>) -> Element {
    let md = use_markdown(&cx);
    let html = md.to_html(cx.props.text);

    cx.render(rsx! {
        div {
            dangerous_inner_html: "{html}",
        }
    })
}

#[derive(Props)]
pub struct MarkdownViewProps<'a> {
    text: &'a str,
}
