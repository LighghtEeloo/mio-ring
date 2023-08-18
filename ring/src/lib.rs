#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use mio_core::MioView;

pub fn App(cx: Scope) -> Element {
    render! {
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

#[inline_props]
fn Home(cx: Scope) -> Element {
    let _view = use_state(cx, || MioView {
        timeline: vec![],
        ring: Default::default(),
    });
    render! {
        div {
        }
    }
}
