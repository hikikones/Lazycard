use dioxus::prelude::*;
use sir::css;

use super::TextSize;

#[derive(Props)]
pub struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<MouseData>>,
    #[props(default)]
    text_size: TextSize,
    #[props(default)]
    border: ButtonBorder<'a>,
    #[props(default)]
    padding: ButtonPadding<'a>,
    #[props(default)]
    disabled: bool,
    #[props(default = "")]
    class: &'a str,
    children: Element<'a>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonBorder<'a> {
    #[default]
    None,
    Rounded,
    Circle,
    Custom(&'a str),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonPadding<'a> {
    None,
    #[default]
    Small,
    Medium,
    Large,
    Custom(&'a str),
}

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    let btn_css = css!(
        "
        display: inline-flex;
        align-items: center;
        white-space: nowrap;
        cursor: pointer;
        border: none;
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
        
        & > * {
            margin-right: 0.25rem;
            pointer-events: none;
        }
        
        & > *:last-child {
            margin-right: 0;
        }
    "
    );

    let border_radius = match cx.props.border {
        ButtonBorder::None => "none",
        ButtonBorder::Rounded => "1.0rem",
        ButtonBorder::Circle => "100%",
        ButtonBorder::Custom(v) => v,
    };

    let padding = match cx.props.padding {
        ButtonPadding::None => "0",
        ButtonPadding::Small => "0.25rem 0.5rem",
        ButtonPadding::Medium => "0.5rem 1.0rem",
        ButtonPadding::Large => "1.0rem 1.5rem",
        ButtonPadding::Custom(v) => v,
    };

    cx.render(rsx! {
        button {
            class: format_args!("{} {}", btn_css, cx.props.class),
            border_radius: border_radius,
            padding: padding,
            font_size: cx.props.text_size.var(),
            disabled: "{cx.props.disabled}",
            onclick: |evt| {
                cx.props.onclick.call(evt);
            },
            &cx.props.children
        }
    })
}
