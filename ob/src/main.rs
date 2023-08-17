use tray_item::{IconSource, TrayItem};

fn main() -> anyhow::Result<()> {
    let mut tray = TrayItem::new("mio-ring", IconSource::Resource("tray-icon-in-rc-file"))?;
    tray.add_label("mio-ring")?;
    #[cfg(target_os = "macos")]
    {
        let mut inner = tray.inner_mut();
        inner.add_quit_item("Quit");
        inner.display();
    }
    loop {}
}
