use super::*;
use itertools::Itertools;
use std::path::Path;

impl EntityKind {
    pub fn synthesize_operations(&self) -> Vec<OperationKind> {
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

impl Actualize for Operation {
    /// an enum dispatch for all
    fn run(&self, mio: &Mio) -> anyhow::Result<()> {
        let base = self.base.clone();
        match self.kind {
            OperationKind::Annotation => {
                let base = base.into_iter().exactly_one()?;
                match mio.specterish(&base).kind() {
                    EntityKind::Image => {
                        todo!()
                    }
                    _ => anyhow::bail!("annotation is not supported for {:?}, image only", base),
                }
            }
            OperationKind::Trim => todo!(),
            OperationKind::Crop => {
                let base = base.into_iter().exactly_one()?;
                match mio.specterish(&base).kind() {
                    EntityKind::Image => {
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
                    _ => anyhow::bail!("crop is not supported for {:?}, image only", base),
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

    impl Operator for CropImage {
        type Source<'a> = &'a Path;
        type Target<'a> = &'a Path;
        fn execute<'a>(self, src: Self::Source<'a>, tar: Self::Target<'a>) -> anyhow::Result<()> {
            let img = image::open(src)?;
            let img = img.crop_imm(self.x, self.y, self.width, self.height);
            img.save(tar)?;
            Ok(())
        }
    }
}

#[cfg(feature = "ocr")]
mod ocr_impl {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct OcrText {
        pub lang: String,
    }

    impl Operator for OcrText {
        type Source<'a> = &'a Path;
        type Target<'a> = &'a Path;
        fn execute<'a>(self, src: Self::Source<'a>, tar: Self::Target<'a>) -> anyhow::Result<()> {
            let mut lt = leptess::LepTess::new(None, self.lang.as_str()).unwrap();
            lt.set_image(src).unwrap();
            let text = lt.get_utf8_text().unwrap();
            std::fs::write(tar, text)?;
            Ok(())
        }
    }
}
