use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn MarkdownView<'a>(cx: Scope<'a, MarkdownViewProps<'a>>) -> Element {
    let html = markdown::to_html(cx.props.text);

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
