use std::collections::HashSet;

use dioxus::prelude::*;

use database::{use_database, TagId};

#[derive(Clone)]
struct Tag {
    id: TagId,
    name: String,
}

#[component]
pub fn Tags(selected: Signal<HashSet<TagId>>) -> Element {
    let db = use_database();
    let tags = use_signal(|| {
        db.fetch_all("SELECT id, name FROM tags", [], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap()
    });

    rsx! {
        div {
            {tags.read().iter().cloned().map(|tag| {
                rsx! {
                    TagButton {
                        key: "{tag.id}",
                        name: tag.name,
                        selected: selected.read().contains(&tag.id),
                        onclick: move |_| {
                            if selected.read().contains(&tag.id) {
                                selected.write().remove(&tag.id);
                            } else {
                                selected.write().insert(tag.id);
                            }
                        },
                    }
                }
            })}
        }
    }
}

#[allow(non_snake_case)]
fn TagButton(props: TagButtonProps) -> Element {
    let color = if props.selected { "blue" } else { "black" };

    rsx! {
        span {
            color: "{color}",
            padding: "5px",
            onclick: move |evt| {
                props.onclick.call(evt);
            },
            "{props.name}"
        },
    }
}

#[derive(Props, PartialEq, Clone)]
struct TagButtonProps {
    name: String,
    selected: bool,
    onclick: EventHandler<Event<MouseData>>,
}
