use dioxus::prelude::*;

pub struct App;

impl App {
    pub fn run(cx: Scope) -> Element {
        cx.render(rsx! {
            div {
                "Hello, world!"
            }
        })
    }
}