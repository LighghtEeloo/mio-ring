use dioxus::prelude::*;

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        mio_thread {}
    })
}

#[derive(PartialEq, Props)]
pub struct MioThread {

}

pub fn mio_thread(cx: Scope<MioThread>) -> Element {
    cx.render(rsx! {
        img {
            src: "https://i.imgur.com/8XvzW0U.png",
        }
    })
}

// use clipboard_master::{CallbackResult, ClipboardHandler, Master};

// struct Handler;

// impl ClipboardHandler for Handler {
//     fn on_clipboard_change(&mut self) -> CallbackResult {
//         let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
//         let contents = ctx.get_contents().unwrap();
//         println!("Clipboard contents: {}", contents);
//         CallbackResult::Next
//     }

//     fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
//         eprintln!("Error: {}", error);
//         CallbackResult::Next
//     }
// }

// pub fn observe_clipboard() -> anyhow::Result<()> {
//     let () = Master::new(Handler).run()?;
//     Ok(())
// }
