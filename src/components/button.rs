use dioxus::prelude::*;
use sir::css;

use super::{Icon, IconName};

#[derive(Props)]
pub struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<MouseData>>,
    #[props(default = "")]
    name: &'a str,
    icon: Option<IconName>,
    #[props(default = "")]
    class: &'a str,
    #[props(default)]
    disabled: bool,
}

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    let icon = cx.props.icon.map(|icon_name| {
        rsx! {
            Icon {
                name: icon_name,
                size: 24,
                fill: "red",
            }
        }
    });

    let css = css!(
        "
        display: inline-block;
        white-space: nowrap;
        text-decoration: none;
        padding: 4px 6px;
        outline: 0;
        border: 0;
        color: black;
        background-color: transparent;

        &:hover {
            background-color: blue;
        }

        &:focus {
            background-color: green;
        }
        
        &:active {
            background-color: red;
        }
        
        &:disabled {
            background-color: black;
        }
    "
    );

    cx.render(rsx! {
        button {
            class: format_args!("{} {}", css, cx.props.class),
            disabled: "{cx.props.disabled}",
            onclick: |evt| {
                if !cx.props.disabled {
                    cx.props.onclick.call(evt);
                }
            },
            icon,
            "{cx.props.name}"
        }
    })
}
