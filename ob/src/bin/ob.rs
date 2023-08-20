use mio_core::{
    Clipboard, CropImage, EntityExt, Interpretable, Mio, MioArchive, MioForce, MioInitiate,
    OcrText, ScreenShot, MioPurge,
};

fn main() -> anyhow::Result<()> {
    // read
    let mut mio = Mio::read_or_bak_with_default();

    // screenshot
    let ids = ScreenShot.interpret(&mut mio)?;
    let del_ids = ids.clone();
    // crop
    let diff = MioInitiate::new(
        CropImage {
            ext: EntityExt::Png,
            x: 100,
            y: 0,
            width: 80,
            height: 40,
        },
        ids,
    )
    .interpret(&mut mio)?;
    let ids = diff.specters.keys().copied().collect();
    let () = MioForce { ids }.interpret(&mio)?;
    // ocr
    let ids = diff.specters.keys().copied().collect();
    let diff = MioInitiate::new(
        OcrText {
            ext: EntityExt::Png,
            lang: "eng".to_string(),
        },
        ids,
    )
    .interpret(&mut mio)?;
    let ids = diff.specters.keys().copied().collect();
    let () = MioForce { ids }.interpret(&mio)?;

    // clipboard
    let clipboard = Clipboard::new()?;
    // let _ids = clipboard.interpret(&mut mio)?;
    let _ = clipboard;

    // archive
    for id in del_ids {
        let archived = MioArchive::Specter(id).interpret(&mut mio)?;
        let _ = archived;
        // eprintln!("{:#?}", archived);
    }

    // purge
    MioPurge.interpret(&mut mio)?;

    // save
    mio.flush()?;
    eprintln!("{:#?}", mio);
    Ok(())
}
