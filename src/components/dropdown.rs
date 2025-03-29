use dioxus::prelude::*;

use crate::components::Button;

use super::{ButtonBorder, ButtonPadding, TextSize};

#[derive(Props, PartialEq, Clone)]
pub struct DropdownProps {
    #[props(default)]
    text_size: TextSize,
    #[props(default)]
    border: ButtonBorder,
    #[props(default)]
    padding: ButtonPadding,
    #[props(default)]
    disabled: bool,

    body: Element,
    children: Element,
}

#[allow(non_snake_case)]
pub fn Dropdown<'a>(props: DropdownProps) -> Element {
    let uuid = use_hook(|| uuid::Uuid::new_v4());

    let on_click = move |_| {
        document::eval(&format!(
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

    rsx! {
        div {
            id: "{uuid}",
            class:"dropdown",

            Button {
                text_size: props.text_size,
                border: props.border,
                padding: props.padding,
                disabled: props.disabled,
                onclick: on_click,
                {props.body}
            }

            div {
                class: "dropdown-content",
                {props.children}
            }
        }
    }
}
