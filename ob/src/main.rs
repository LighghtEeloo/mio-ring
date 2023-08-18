use mio_core::{Mio, ScreenShot};
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
    let mut mio = Mio::read_or_bak_with_default();
    mio.register(&mut ScreenShot)?;
    mio.flush()?;
    loop {}
}
