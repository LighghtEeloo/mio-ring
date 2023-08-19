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
        ul {
            for eph in view.timeline.iter() {
                li {
                    "{format_time(eph.time)}"
                    div {
                        SpecterEntry {
                            dirs: &mio.dirs,
                            id: eph.base,
                            ring: &view.ring,
                        }
                    }
                }
            }
        }
    }
}

#[inline_props]
fn SpecterEntry<'a>(cx: Scope, dirs: &'a MioDirs, id: MioId, ring: &'a MioRing) -> Element {
    let specter = ring.specterish(&id);
    let path = specter.locate(dirs);
    let kind = specter.kind();
    let ops = kind.synthesize();
    let ops_group = rsx!(
        OperationButtonGroup {
            ops: ops
        }
    );
    match kind {
        EntityKind::Text => {
            let text = std::fs::read_to_string(&path).unwrap();
            render!(div {
                style: "border: 1px black solid; padding: 5px;",
                pre {
                    "{text}"
                }
                ops_group
            })
        }
        EntityKind::Image => render!(div {
            style: "border: 1px black solid; padding: 5px;",
            img {
                style: "max-height: 120px;",
                src: "{path.display()}",
            }
            ops_group
        }),
        EntityKind::Audio => todo!(),
        EntityKind::Video => todo!(),
    }
}

#[inline_props]
fn OperationButtonGroup(cx: Scope, ops: Vec<OperationKind>) -> Element {
    render! {
        div {
            for op in ops {
                OperationButton {
                    op: op
                }
            }
        }
    }
}

#[inline_props]
fn OperationButton<'a>(cx: Scope, op: &'a OperationKind) -> Element {
    render! {
        button {
            "{op:?}"
        }
    }
}