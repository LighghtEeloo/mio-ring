use super::*;

impl EntityKind {
    pub fn synthesize(&self) -> Vec<OperationKind> {
        let mut v = Vec::new();
        match self {
            EntityKind::Text => {}
            EntityKind::Image => {
                #[cfg(feature = "image")]
                {
                    v.push(OperationKind::Annotation);
                    v.push(OperationKind::Crop);
                }
                #[cfg(feature = "ocr")]
                {
                    v.push(OperationKind::As(EntityKind::Text));
                }
            }
            EntityKind::Audio => {}
            EntityKind::Video => {}
        }
        v
    }
}

impl OperationKind {
    pub fn analyze(&self, src: impl IntoIterator<Item = EntityKind>) -> anyhow::Result<EntityKind> {
        let src = src.into_iter().collect::<Vec<_>>();
        match (src.as_slice(), self) {
            #[cfg(feature = "image")]
            (&[EntityKind::Image], OperationKind::Crop) => Ok(EntityKind::Image),
            #[cfg(feature = "image")]
            (&[EntityKind::Image], OperationKind::Annotation) => Ok(EntityKind::Image),
            #[cfg(feature = "ocr")]
            (&[EntityKind::Image], OperationKind::As(EntityKind::Text)) => Ok(EntityKind::Text),
            _ => Err(anyhow::anyhow!(
                "operation {:?} is not supported for {:?}",
                self,
                src
            )),
        }
    }
}

impl Actualizable for Operation {
    /// an enum dispatch for all
    fn run(&self, mio: &Mio) -> anyhow::Result<()> {
        let base = self.base.clone();
        let _res_kind = self
            .kind
            .analyze(base.iter().map(|base| mio.specterish(&base).kind()))?;
        match self.kind {
            OperationKind::Annotation => {
                let _base = base.into_iter().exactly_one()?;
                todo!()
            }
            OperationKind::Trim => todo!(),
            OperationKind::Crop => {
                let base = base.into_iter().exactly_one()?;
                #[cfg(feature = "image")]
                {
                    // ensure that the base is actualized
                    mio.specterish(&base).run(mio)?;
                    image_impl::CropImage::prepare(self)?.execute(
                        &mio.specterish(&base).locate(&mio.dirs).as_path(),
                        &mio.specterish(&self.specter).locate(&mio.dirs).as_path(),
                    )
                }
                #[cfg(not(feature = "image"))]
                {
                    anyhow::bail!("feature `image` is not enabled")
                }
            }
            OperationKind::Resize => todo!(),
            OperationKind::As(EntityKind::Text) => {
                let base = base.into_iter().exactly_one()?;
                match mio.specterish(&base).kind() {
                    EntityKind::Text => Ok(()),
                    EntityKind::Image => {
                        #[cfg(feature = "ocr")]
                        {
                            // ensure that the base is actualized
                            mio.specterish(&base).run(mio)?;
                            ocr_impl::OcrText::prepare(self)?.execute(
                                &mio.specterish(&base).locate(&mio.dirs).as_path(),
                                &mio.specterish(&self.specter).locate(&mio.dirs).as_path(),
                            )
                        }
                        #[cfg(not(feature = "ocr"))]
                        {
                            anyhow::bail!("feature `ocr` is not enabled")
                        }
                    }
                    EntityKind::Audio => anyhow::bail!("audio to text is not supported"),
                    EntityKind::Video => anyhow::bail!("video to text is not supported"),
                }
            }
            OperationKind::As(_) => todo!(),
            OperationKind::Summarize => todo!(),
        }
    }
}

#[cfg(feature = "image")]
mod image_impl {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct CropImage {
        pub x: u32,
        pub y: u32,
        pub width: u32,
        pub height: u32,
    }

    impl Operable for CropImage {
        type Source<'a> = &'a Path;
        type Target<'a> = &'a Path;
        fn kind(&self) -> OperationKind {
            OperationKind::Crop
        }
        fn execute<'a>(self, src: Self::Source<'a>, tar: Self::Target<'a>) -> anyhow::Result<()> {
            let img = image::open(src)?;
            let img = img.crop_imm(self.x, self.y, self.width, self.height);
            img.save(tar)?;
            Ok(())
        }
    }
}
#[cfg(feature = "image")]
pub use image_impl::*;

#[cfg(feature = "ocr")]
mod ocr_impl {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct OcrText {
        pub lang: String,
    }

    impl Operable for OcrText {
        type Source<'a> = &'a Path;
        type Target<'a> = &'a Path;
        fn kind(&self) -> OperationKind {
            OperationKind::As(EntityKind::Text)
        }
        fn execute<'a>(self, src: Self::Source<'a>, tar: Self::Target<'a>) -> anyhow::Result<()> {
            let mut lt = leptess::LepTess::new(None, self.lang.as_str()).unwrap();
            lt.set_image(src).unwrap();
            let text = lt.get_utf8_text().unwrap();
            std::fs::write(tar, text)?;
            Ok(())
        }
    }
}
#[cfg(feature = "ocr")]
pub use ocr_impl::*;