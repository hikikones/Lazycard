use std::collections::HashSet;

use dioxus::prelude::*;

use database::{use_database, TagId};

#[derive(Clone)]
struct Tag {
    id: TagId,
    name: String,
}

#[allow(non_snake_case)]
pub fn Tags<'a>(cx: Scope<'a, TagsProps>) -> Element<'a> {
    let db = use_database(&cx);
    let tags = use_ref(&cx, || {
        db.fetch_all("SELECT id, name FROM tags", [], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap()
    });

    cx.render(rsx! {
        div {
            tags.read().iter().cloned().map(|tag| {
                rsx! {
                    TagButton {
                        key: "{tag.id}",
                        name: tag.name,
                        selected: cx.props.selected.read().contains(&tag.id),
                        onclick: move |_| {
                            if cx.props.selected.read().contains(&tag.id) {
                                cx.props.selected.write().remove(&tag.id);
                            } else {
                                cx.props.selected.write().insert(tag.id);
                            }
                        },
                    }
                }
            })
        }
    })
}

#[derive(Props)]
pub struct TagsProps<'a> {
    selected: &'a UseRef<HashSet<TagId>>,
}

#[allow(non_snake_case)]
fn TagButton<'a>(cx: Scope<'a, TagButtonProps>) -> Element<'a> {
    let color = if cx.props.selected { "blue" } else { "black" };

    cx.render(rsx! {
        span {
            color: "{color}",
            padding: "5px",
            onclick: |evt| {
                cx.props.onclick.call(evt);
            },
            "{cx.props.name}"
        },
    })
}

#[derive(Props)]
struct TagButtonProps<'a> {
    name: String,
    selected: bool,
    onclick: EventHandler<'a, Event<MouseData>>,
}
