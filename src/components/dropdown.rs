use dioxus::prelude::*;
use dioxus_desktop::use_eval;
use sir::{css, global_css};

#[derive(Props)]
pub struct DropdownProps<'a> {
    name: Option<&'a str>,
    #[props(default)]
    disabled: bool,
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

    let name = cx.props.name.map(|name| {
        rsx! {
            span {
                pointer_events: "none",
                "{name}"
            }
        }
    });

    cx.render(rsx! {
        div {
            id: "{uuid}",
            class: css!("
                position: relative;
            "),

            button {
                disabled: "{cx.props.disabled}",
                onclick: on_click,
                name,
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
