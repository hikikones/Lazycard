use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn Load(cx: Scope) -> Element {
    let router = use_router(&cx);
    let has_loaded = use_state(&cx, || false);

    has_loaded.then(|| {
        router.push_route("/review", None, None);
    });

    use_effect(&cx, has_loaded, |has_loaded| async move {
        has_loaded.set(true);
    });

    cx.render(rsx! {
        h1 { "Load" }
    })
}
