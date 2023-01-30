use dioxus::prelude::*;
use sir::css;

#[derive(Props)]
pub struct ButtonProps<'a> {
    onclick: EventHandler<'a, Event<MouseData>>,
    #[props(default)]
    disabled: bool,
    #[props(default = "")]
    class: &'a str,
    children: Element<'a>,
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

        & > svg {
            fill: var(--secondary-text-color);
        }

        & > svg + span {
            margin-left: 0.25rem;
        }

        & > * {
            pointer-events: none;   
        }
    "
    );

    cx.render(rsx! {
        button {
            class: format_args!("{} {}", btn_css, cx.props.class),
            disabled: "{cx.props.disabled}",
            onclick: |evt| {
                cx.props.onclick.call(evt);
            },
            &cx.props.children
        }
    })
}
