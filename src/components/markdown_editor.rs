use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn MarkdownEditor<'a>(cx: Scope<'a, MarkdownEditorProps<'a>>) -> Element {
    cx.render(rsx! {
        textarea {
            rows: "10",
            cols: "80",
            value: "{cx.props.value}",
            oninput: |evt| {
                cx.props.oninput.call(evt.value.clone());
            },
        }
    })
}

#[derive(Props)]
pub struct MarkdownEditorProps<'a> {
    value: &'a str,
    oninput: EventHandler<'a, String>,
}
