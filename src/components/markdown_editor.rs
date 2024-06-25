use std::path::Path;

use dioxus::prelude::*;

use database::*;

#[component]
pub fn MarkdownEditor(text: Signal<String>) -> Element {
    rsx! {
        div {
            button {
                onclick: move |_| {
                    if let Some(file) = &rfd::FileDialog::new()
                        .add_filter("Image", &["jpeg", "jpg", "png"])
                        .pick_file()
                    {
                        let ext = file.extension().unwrap().to_string_lossy();
                        let bytes = std::fs::read(file).unwrap();
                        let hash = Seahash::from(bytes.as_ref());

                        let asset_name = &format!("{}.{}", hash, ext.to_lowercase());
                        let asset_path = &Path::new(config::ASSETS_DIR).join(asset_name);

                        if !asset_path.exists() {
                            std::fs::copy(file, asset_path).unwrap();
                        }

                        text.write().push_str(&format!("\n![]({asset_name})\n"));
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
                oninput: move |event| {
                    text.set(event.value().clone());
                },
            }
        }
    }
}
