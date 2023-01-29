use dioxus::prelude::*;
use sir::css;

use super::{Icon, IconName, IconSize};

#[derive(Props)]
pub struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<MouseData>>,
    name: Option<&'a str>,
    icon: Option<IconName>,
    #[props(default)]
    icon_size: IconSize,
    #[props(default = "")]
    class: &'a str,
    #[props(default)]
    disabled: bool,
}

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    let btn_css = css!(
        "
        display: inline-flex;
        align-items: center;
        cursor: pointer;
        border: none;
        line-height: 0;
        padding: 0.25rem;
        color: var(--secondary-text-color);
        background-color: transparent;

        &:hover {
            background-color: var(--secondary-color);
        }
        
        &:active {
            background-color: var(--secondary-variant-color);
        }
        
        &:disabled {
            color: var(--secondary-text-color);
            background-color: transparent;
            cursor: unset;
            opacity: 0.5;
        }

        & > svg {
            fill: var(--secondary-text-color);
        }

        & > svg + span {
            margin-left: 0.25rem;
        }
    "
    );

    let name = cx.props.name.map(|name| {
        rsx! {
            span { "{name}" }
        }
    });

    let icon = cx.props.icon.map(|icon| {
        rsx! {
            Icon {
                name: icon,
                size: cx.props.icon_size,
            }
        }
    });

    cx.render(rsx! {
        button {
            class: format_args!("{} {}", btn_css, cx.props.class),
            disabled: "{cx.props.disabled}",
            onclick: |evt| {
                if !cx.props.disabled {
                    cx.props.onclick.call(evt);
                }
            },
            icon,
            name,
        }
    })
}
