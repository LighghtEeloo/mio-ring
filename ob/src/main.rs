use tray_item::{TrayItem, IconSource};

fn main() -> anyhow::Result<()> {
    let mut tray = TrayItem::new("mio-ring", IconSource::Resource("tray-icon-in-rc-file"))?;
    tray.add_label("mio-ring")?;
    Ok(())
}
