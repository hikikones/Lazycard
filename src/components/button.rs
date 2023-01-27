use dioxus::prelude::*;
use sir::css;

use super::{Icon, IconName};

#[derive(Props)]
pub struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<MouseData>>,
    #[props(default = "")]
    name: &'a str,
    icon: Option<IconName>,
    #[props(default)]
    size: ButtonSize,
    #[props(default = "")]
    class: &'a str,
    #[props(default)]
    disabled: bool,
}

#[derive(Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
    XL,
    XXL,
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
        padding: 4px 6px;
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
    "
    );

    let size = match cx.props.size {
        ButtonSize::Small => 20,
        ButtonSize::Medium => 24,
        ButtonSize::Large => 28,
        ButtonSize::XL => 32,
        ButtonSize::XXL => 64,
    };

    let icon = cx.props.icon.map(|icon| {
        rsx! {
            Icon {
                name: icon,
                size: size,
                fill: "var(--secondary-text-color)",
                class: css!("margin-right: 3px;"),
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
            "{cx.props.name}"
        }
    })
}
