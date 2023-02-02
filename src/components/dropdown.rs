use dioxus::prelude::*;
use dioxus_desktop::use_eval;
use sir::{css, global_css};

use crate::components::Button;

use super::{ButtonBorder, ButtonPadding, TextSize};

#[derive(Props)]
pub struct DropdownProps<'a> {
    #[props(default)]
    text_size: TextSize,
    #[props(default)]
    border: ButtonBorder<'a>,
    #[props(default)]
    padding: ButtonPadding<'a>,
    #[props(default)]
    disabled: bool,

    body: Element<'a>,
    children: Element<'a>,
}

#[allow(non_snake_case)]
pub fn Dropdown<'a>(cx: Scope<'a, DropdownProps<'a>>) -> Element {
    let eval = use_eval(&cx);
    let uuid = *cx.use_hook(|| uuid::Uuid::new_v4());

    global_css!(
        "
        .show-dropdown {
            visibility: visible !important;
        }
    "
    );

    let on_click = move |_| {
        eval(format!(
            r#"
            const dropdown = document.getElementById('{uuid}');
            const menu = dropdown.children[1];
            menu.classList.toggle('show-dropdown');
            document.documentElement.addEventListener('click', (event) => {{
                if (event.target.parentElement.id !== '{uuid}') {{
                    menu.classList.remove('show-dropdown');
                }}
            }}, {{once: true}});
        "#
        ));
    };

    cx.render(rsx! {
        div {
            id: "{uuid}",
            class: css!("
                display: inline-block;
                position: relative;
            "),

            Button {
                text_size: cx.props.text_size,
                border: cx.props.border,
                padding: cx.props.padding,
                disabled: cx.props.disabled,
                onclick: on_click,
                &cx.props.body,
            }

            div {
                class: css!("
                    visibility: hidden;
                    position: absolute;
                    right: 0;
                    z-index: 2;
                    display: flex;
                    flex-direction: column;
                    background-color: var(--surface-color);
                "),
                &cx.props.children
            }
        }
    })
}
