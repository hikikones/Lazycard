use std::path::Path;

use dioxus::prelude::*;

use database::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn MarkdownEditor<'a>(cx: Scope, text: &'a UseState<String>) -> Element {
    cx.render(rsx! {
        div {
            button {
                onclick: |_| {
                    if let Some(file) = &rfd::FileDialog::new()
                        .add_filter("Image", &["jpeg", "jpg", "png"])
                        .pick_file()
                    {
                        let ext = file.extension().unwrap().to_string_lossy();
                        let bytes = std::fs::read(file).unwrap();
                        let hash = Seahash::from(bytes.as_ref());

                        let asset_name = &format!("{}.{}", hash.raw(), ext.to_lowercase());
                        let asset_path = &Path::new(config::ASSETS_DIR).join(asset_name);

                        if !asset_path.exists() {
                            std::fs::copy(file, asset_path).unwrap();
                        }

                        text.make_mut().push_str(&format!("\n![]({asset_name})\n"));
                    }
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
