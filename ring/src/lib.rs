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