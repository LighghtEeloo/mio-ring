use super::*;

impl Mio {
    /// run a persistable and memorize its entities into the mio ring
    fn register(&mut self, persister: &impl Persistable) -> anyhow::Result<Vec<MioId>> {
        let mut ids = Vec::new();
        for (src, ext) in persister.persist()? {
            let id = self.alloc.allocate().into();
            let entity = Specter {
                id,
                ext,
                nonce: Specter::<Concrete>::gen_nouce(),
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
        fn persist(&self) -> anyhow::Result<Vec<(NamedTempFile, EntityExt)>> {
            let mut v = Vec::new();
            for screen in screenshots::Screen::all()?.into_iter() {
                let image = screen.capture()?.to_png(None)?;
                let mut file = NamedTempFile::new()?;
                file.write_all(image.as_slice())?;
                v.push((file, EntityExt::Png));
            }
            Ok(v)
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
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    pub struct Clipboard {
        board: Rc<RefCell<arboard::Clipboard>>,
    }

    impl Clipboard {
        pub fn new() -> anyhow::Result<Self> {
            Ok(Self {
                board: Rc::new(RefCell::new(arboard::Clipboard::new()?)),
            })
        }
    }
    impl Default for Clipboard {
        fn default() -> Self {
            Self::new().unwrap()
        }
    }

    impl Persistable for Clipboard {
        fn persist(&self) -> anyhow::Result<Vec<(NamedTempFile, EntityExt)>> {
            let mut board = self.board.borrow_mut();
            if let Ok(contents) = board.get_text() {
                let mut file = NamedTempFile::new()?;
                file.write_all(contents.as_bytes())?;
                Ok(vec![(file, EntityExt::Txt)])
            } else if let Ok(contents) = board.get_image() {
                use image::{DynamicImage, ImageBuffer, ImageOutputFormat};
                let mut file = NamedTempFile::new()?;
                let buf = ImageBuffer::from_raw(
                    contents.width as u32,
                    contents.height as u32,
                    contents.bytes.into_owned(),
                )
                .ok_or_else(|| anyhow::anyhow!("failed to decode clipboard image"))?;
                DynamicImage::ImageRgba8(buf).write_to(&mut file, ImageOutputFormat::Png)?;
                Ok(vec![(file, EntityExt::Png)])
            } else {
                log::warn!("clipboard is empty or has unsupported content");
                Ok(Vec::new())
            }
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
