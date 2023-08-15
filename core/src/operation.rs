use super::*;
use itertools::Itertools;
use std::path::Path;

impl Actualize for Operation {
    /// an enum dispatch for all
    fn run(&self, mio: &Mio) -> anyhow::Result<()> {
        let base = self.base.clone();
        match self.kind {
            OperationKind::Annotation => {
                let base = base.into_iter().exactly_one()?;
                let kind = mio.kind(&base);
                kind.among([EntityKind::Image])?;
                todo!()
            }
            OperationKind::Trim => todo!(),
            OperationKind::Crop => {
                let base = base.into_iter().exactly_one()?;
                let kind = mio.kind(&base);
                kind.among([EntityKind::Image])?;
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

    impl Morphism for CropImage {
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
