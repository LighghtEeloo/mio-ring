#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use mio_core::*;
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
                Specter {
                    dirs: &mio.dirs,
                    id: eph.base,
                    ring: &view.ring,
                }
            }
        }
    }
}

#[inline_props]
fn Specter<'a>(cx: Scope, dirs: &'a MioDirs, id: MioId, ring: &'a MioRing) -> Element {
    let specter = ring.specterish(&id);
    let kind = specter.kind();
    let path = specter.locate(dirs);
    match kind {
        EntityKind::Text => {
            let text = std::fs::read_to_string(&path).unwrap();
            render!(pre {
                "{text}"
            })
        }
        EntityKind::Image => render!(img {
            src: "{path.display()}",
            height: "60px",
        }),
        EntityKind::Audio => todo!(),
        EntityKind::Video => todo!(),
    }
}
