use dioxus::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconName {
    Drafts,
    Layers,
    AddCircle,
    AddBox,
    Dashboard,
    Settings,
    Done,
    Close,
    DoubleArrow,
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum IconSize {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Gigantic,
    Custom(u32),
}

#[derive(Props, PartialEq)]
pub struct IconProps<'a> {
    name: IconName,
    #[props(default)]
    size: IconSize,
    #[props(default = "black")]
    fill: &'a str,
    #[props(default = "")]
    class: &'a str,
}

#[allow(non_snake_case)]
pub fn Icon<'a>(cx: Scope<'a, IconProps<'a>>) -> Element {
    let size = match cx.props.size {
        IconSize::Tiny => 16,
        IconSize::Small => 20,
        IconSize::Medium => 24,
        IconSize::Large => 36,
        IconSize::Gigantic => 48,
        IconSize::Custom(v) => v,
    };

    cx.render(rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            fill: "{cx.props.fill}",
            class: "{cx.props.class}",
            view_box: "0 0 48 48",

            path {
                d: cx.props.name.d(),
            }
        }
    })
}

impl IconName {
    // https://fonts.google.com/icons?icon.style=Outlined&icon.platform=web
    fn d(&self) -> &str {
        match self {
            IconName::Drafts => "m24 2 18.55 11.1q.85.45 1.15 1.225.3.775.3 1.525V39q0 1.2-.9 2.1-.9.9-2.1.9H7q-1.2 0-2.1-.9Q4 40.2 4 39V15.85q0-.75.325-1.525.325-.775 1.125-1.225Zm0 23.3 16.8-9.85L24 5.35 7.2 15.45Z",
            IconName::Layers => "m24 41.5-18-14 2.5-1.85L24 37.7l15.5-12.05L42 27.5Zm0-7.6-18-14 18-14 18 14Z",
            IconName::AddCircle => "M22.65 34h3v-8.3H34v-3h-8.35V14h-3v8.7H14v3h8.65ZM24 44q-4.1 0-7.75-1.575-3.65-1.575-6.375-4.3-2.725-2.725-4.3-6.375Q4 28.1 4 23.95q0-4.1 1.575-7.75 1.575-3.65 4.3-6.35 2.725-2.7 6.375-4.275Q19.9 4 24.05 4q4.1 0 7.75 1.575 3.65 1.575 6.35 4.275 2.7 2.7 4.275 6.35Q44 19.85 44 24q0 4.1-1.575 7.75-1.575 3.65-4.275 6.375t-6.35 4.3Q28.15 44 24 44Z",
            IconName::AddBox => "M22.5 34h3v-8.5H34v-3h-8.5V14h-3v8.5H14v3h8.5ZM9 42q-1.2 0-2.1-.9Q6 40.2 6 39V9q0-1.2.9-2.1Q7.8 6 9 6h30q1.2 0 2.1.9.9.9.9 2.1v30q0 1.2-.9 2.1-.9.9-2.1.9Z",
            IconName::Dashboard => "M25.5 19.5V6H42v13.5ZM6 25.5V6h16.5v19.5ZM25.5 42V22.5H42V42ZM6 42V28.5h16.5V42Z",
            IconName::Settings => "m19.4 44-1-6.3q-.95-.35-2-.95t-1.85-1.25l-5.9 2.7L4 30l5.4-3.95q-.1-.45-.125-1.025Q9.25 24.45 9.25 24q0-.45.025-1.025T9.4 21.95L4 18l4.65-8.2 5.9 2.7q.8-.65 1.85-1.25t2-.9l1-6.35h9.2l1 6.3q.95.35 2.025.925Q32.7 11.8 33.45 12.5l5.9-2.7L44 18l-5.4 3.85q.1.5.125 1.075.025.575.025 1.075t-.025 1.05q-.025.55-.125 1.05L44 30l-4.65 8.2-5.9-2.7q-.8.65-1.825 1.275-1.025.625-2.025.925l-1 6.3ZM24 30.5q2.7 0 4.6-1.9 1.9-1.9 1.9-4.6 0-2.7-1.9-4.6-1.9-1.9-4.6-1.9-2.7 0-4.6 1.9-1.9 1.9-1.9 4.6 0 2.7 1.9 4.6 1.9 1.9 4.6 1.9Z",
            IconName::Done => "M18.9 35.7 7.7 24.5l2.15-2.15 9.05 9.05 19.2-19.2 2.15 2.15Z",
            IconName::Close => "m12.45 37.65-2.1-2.1L21.9 24 10.35 12.45l2.1-2.1L24 21.9l11.55-11.55 2.1 2.1L26.1 24l11.55 11.55-2.1 2.1L24 26.1Z",
            IconName::DoubleArrow => "m12.1 38 10.5-14-10.5-14h3.7l10.5 14-10.5 14Zm12.6 0 10.5-14-10.5-14h3.7l10.5 14-10.5 14Z",
        }
    }
}
