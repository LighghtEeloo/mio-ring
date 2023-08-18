#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use mio_core::{Mio, MioView};
use std::time::SystemTime;

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
    let mio = use_state(cx, || Mio::read_or_bak_with_default());
    let view = use_state(cx, || MioView::all(&mio));

    fn format_time(t: SystemTime) -> String {
        let dt = chrono::DateTime::<chrono::Local>::from(t);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    render! {
        for eph in view.timeline.iter() {
            li {
                "{format_time(eph.time)}"
                img {
                    src: "{view.ring.specterish(&eph.base).locate(&mio.dirs).display()}",
                    height: "60px",
                }
            }
        }
    }
}
