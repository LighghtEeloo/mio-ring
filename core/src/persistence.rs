use super::*;

#[cfg(feature = "screenshot")]
mod screenshot_impl {
    use super::*;

    pub struct ScreenShot;

    impl Persistable for ScreenShot {
        fn persist(&self) -> anyhow::Result<Vec<(PathBuf, EntityExt)>> {
            let screen = screenshots::Screen::all()?.into_iter().exactly_one()?;
            let image = screen.capture()?.to_png(None)?;
            let mut file = tempfile::NamedTempFile::new()?;
            file.write_all(image.as_slice())?;
            Ok(vec![(file.path().to_path_buf(), EntityExt::Png)])
        }
    }
}
#[cfg(feature = "screenshot")]
pub use screenshot_impl::ScreenShot;

#[cfg(feature = "clipboard")]
mod clipboard_impl {
    use super::*;

    use clipboard::ClipboardContext;
    use clipboard::ClipboardProvider;

    pub struct Clipboard;

    impl Persistable for Clipboard {
        fn persist(&self) -> anyhow::Result<Vec<(PathBuf, EntityExt)>> {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let contents = ctx.get_contents().unwrap();
            let mut file = tempfile::NamedTempFile::new()?;
            file.write_all(contents.as_bytes())?;
            Ok(vec![(file.path().to_path_buf(), EntityExt::Txt)])
        }
    }
}
#[cfg(feature = "clipboard")]
pub use clipboard_impl::Clipboard;
