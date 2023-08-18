use mio_core::{
    Clipboard, CropImage, Interpretable, Mio, MioForce, MioInitiate, OcrText, ScreenShot,
};
// use tray_item::{IconSource, TrayItem};

fn main() -> anyhow::Result<()> {
    // #[cfg(not(target_os = "macos"))]
    // {
    //     let mut tray = TrayItem::new("mio-ring", IconSource::Resource("tray-icon-in-rc-file"))?;
    //     tray.add_label("mio-ring")?;
    // }
    // #[cfg(target_os = "macos")]
    // {
    //     let mut tray = TrayItem::new("澪", IconSource::Resource(""))?;
    //     // let mut tray = TrayItem::new("澪", IconSource::Resource("tray.icns"))?;
    //     tray.add_label("mio-ring")?;
    //     let inner = tray.inner_mut();
    //     inner.add_quit_item("Quit");
    //     inner.display();
    // }

    // read
    let mut mio = Mio::read_or_bak_with_default();

    // screenshot
    let ids = ScreenShot.interpret(&mut mio)?;
    // crop
    let diff = MioInitiate::new(
        CropImage {
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
            lang: "eng".to_string(),
        },
        ids,
    )
    .interpret(&mut mio)?;
    let ids = diff.specters.keys().copied().collect();
    let () = MioForce { ids }.interpret(&mio)?;

    // clipboard
    let clipboard = Clipboard::new()?;
    let _ids = clipboard.interpret(&mut mio)?;

    // save
    mio.flush()?;
    eprintln!("{:#?}", mio);
    Ok(())
}
