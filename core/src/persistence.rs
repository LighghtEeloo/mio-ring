use super::*;

#[cfg(feature = "screenshot")]
mod screenshot_impl {
    use super::*;

    pub struct ScreenShot;

    impl Persistable for ScreenShot {
        fn persist(&mut self) -> anyhow::Result<Vec<(PathBuf, EntityExt)>> {
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

    pub struct Clipboard {
        board: arboard::Clipboard,
    }

    impl Clipboard {
        pub fn new() -> anyhow::Result<Self> {
            Ok(Self {
                board: arboard::Clipboard::new()?,
            })
        }
    }
    impl Default for Clipboard {
        fn default() -> Self {
            Self::new().unwrap()
        }
    }

    impl Persistable for Clipboard {
        fn persist(&mut self) -> anyhow::Result<Vec<(PathBuf, EntityExt)>> {
            let contents = self.board.get_text().unwrap();
            let mut file = tempfile::NamedTempFile::new()?;
            file.write_all(contents.as_bytes())?;
            Ok(vec![(file.path().to_path_buf(), EntityExt::Txt)])
        }
    }
}
#[cfg(feature = "clipboard")]
pub use clipboard_impl::Clipboard;
