use std::path::Path;

use dioxus::prelude::*;

use database::*;

use crate::hooks::use_config;

#[allow(non_snake_case)]
#[inline_props]
pub fn MarkdownEditor<'a>(cx: Scope, text: &'a UseState<String>) -> Element {
    let cfg = use_config(&cx);

    cx.render(rsx! {
        div {
            button {
                onclick: |_| {
                    let file = &rfd::FileDialog::new()
                        .add_filter("Image", &["jpeg", "jpg", "png"])
                        .pick_file()
                        .unwrap();
                    let ext = file.extension().unwrap().to_string_lossy();
                    let bytes = std::fs::read(file).unwrap();
                    let hash = Seahash::from(bytes.as_ref());

                    let assets_dir = cfg.borrow().assets_dir();
                    let asset_name = &format!("{}.{}", hash.raw(), ext);
                    let asset_path = &Path::new(assets_dir).join(asset_name);

                    if !asset_path.exists() {
                        std::fs::copy(file, asset_path).unwrap();
                    }

                    text.make_mut().push_str(&format!("\n![]({}/{})\n", assets_dir, asset_name));
                },
                "Image"
            }
        }
        div {
            textarea {
                rows: "10",
                cols: "60",
                value: "{text}",
                oninput: |event| {
                    text.set(event.value.clone());
                },
            }
        }
    })
}
