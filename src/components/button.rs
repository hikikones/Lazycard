use dioxus::prelude::*;

use super::TextSize;

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    onclick: EventHandler<Event<MouseData>>,
    #[props(default)]
    text_size: TextSize,
    #[props(default)]
    border: ButtonBorder,
    #[props(default)]
    padding: ButtonPadding,
    #[props(default)]
    disabled: bool,
    #[props(default = "")]
    class: &'static str,
    children: Element,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonBorder {
    #[default]
    None,
    Rounded,
    Circle,
    Custom(&'static str),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonPadding {
    None,
    #[default]
    Small,
    Medium,
    Large,
    Custom(&'static str),
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let border_radius = match props.border {
        ButtonBorder::None => "none",
        ButtonBorder::Rounded => "1.0rem",
        ButtonBorder::Circle => "100%",
        ButtonBorder::Custom(v) => v,
    };

    let padding = match props.padding {
        ButtonPadding::None => "0",
        ButtonPadding::Small => "0.25rem 0.5rem",
        ButtonPadding::Medium => "0.5rem 1.0rem",
        ButtonPadding::Large => "1.0rem 1.5rem",
        ButtonPadding::Custom(v) => v,
    };

    rsx! {
        button {
            class: props.class,
            border_radius: border_radius,
            padding: padding,
            font_size: props.text_size.var(),
            disabled: props.disabled,
            onclick: move |evt| {
                props.onclick.call(evt);
            },
            {props.children}
        }
    }
}
