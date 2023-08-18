use super::*;

impl Mio {
    /// run a persistable and memorize its entities into the mio ring
    fn register(&mut self, persister: &mut impl Persistable) -> anyhow::Result<Vec<MioId>> {
        let mut ids = Vec::new();
        for (src, ext) in persister.persist()? {
            let id = self.alloc.allocate().into();
            let entity = Specter {
                id,
                ext,
                deps: Vec::new(),
                body: Concrete {
                    pool: self.alloc.allocate_pool(POOL_SIZE),
                    providence: Providence::Registered,
                },
            };
            self.chronology.push(Ephemerality {
                time: SystemTime::now(),
                base: id,
            });
            entity
                .ring_and(&mut self.ring)?
                .replace(&self.dirs, src.path())?;
            ids.push(id);
        }
        Ok(ids)
    }
}

#[cfg(feature = "screenshot")]
mod screenshot_impl {
    use super::*;

    pub struct ScreenShot;

    impl Persistable for ScreenShot {
        fn persist(&mut self) -> anyhow::Result<Vec<(NamedTempFile, EntityExt)>> {
            let screen = screenshots::Screen::all()?.into_iter().exactly_one()?;
            let image = screen.capture()?.to_png(None)?;
            let mut file = NamedTempFile::new()?;
            file.write_all(image.as_slice())?;
            Ok(vec![(file, EntityExt::Png)])
        }
    }

    impl Interpretable for ScreenShot {
        type Mio<'a> = &'a mut Mio;
        type Target<'a> = Vec<MioId>;
        fn interpret<'a>(mut self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
            mio.register(&mut self)
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
        fn persist(&mut self) -> anyhow::Result<Vec<(NamedTempFile, EntityExt)>> {
            let contents = self.board.get_text().unwrap();
            let mut file = tempfile::NamedTempFile::new()?;
            file.write_all(contents.as_bytes())?;
            Ok(vec![(file, EntityExt::Txt)])
        }
    }

    impl Interpretable for Clipboard {
        type Mio<'a> = &'a mut Mio;
        type Target<'a> = Vec<MioId>;
        fn interpret<'a>(mut self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
            mio.register(&mut self)
        }
    }
}
#[cfg(feature = "clipboard")]
pub use clipboard_impl::Clipboard;
