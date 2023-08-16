use itertools::Itertools;
use std::{
    io::{self, Write},
    path::PathBuf,
};

#[cfg(feature = "screenshot")]
mod screenshot_impl {
    use super::*;

    pub fn screenshot() -> anyhow::Result<PathBuf> {
        let screen = screenshots::Screen::all()?.into_iter().exactly_one()?;
        let image = screen.capture()?.to_png(None)?;
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(image.as_slice())?;
        Ok(file.path().to_path_buf())
    }
}
#[cfg(feature = "screenshot")]
pub use screenshot_impl::screenshot;

#[cfg(feature = "clipboard")]
mod clipboard_impl {
    use super::*;

    use clipboard::ClipboardContext;
    use clipboard::ClipboardProvider;
    use clipboard_master::{CallbackResult, ClipboardHandler, Master};

    pub fn clipboard() -> anyhow::Result<PathBuf> {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        let contents = ctx.get_contents().unwrap();
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(contents.as_bytes())?;
        Ok(file.path().to_path_buf())
    }

    struct Handler;

    impl ClipboardHandler for Handler {
        fn on_clipboard_change(&mut self) -> CallbackResult {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let contents = ctx.get_contents().unwrap();
            println!("Clipboard contents: {}", contents);
            CallbackResult::Next
        }

        fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
            eprintln!("Error: {}", error);
            CallbackResult::Next
        }
    }

    pub fn observe_clipboard() -> anyhow::Result<()> {
        let () = Master::new(Handler).run()?;
        Ok(())
    }
}
#[cfg(feature = "clipboard")]
pub use clipboard_impl::{clipboard, observe_clipboard};
