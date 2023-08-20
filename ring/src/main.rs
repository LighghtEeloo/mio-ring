use dioxus_desktop::{Config, WindowBuilder};
use mio_ring::{App, TempAssets};

fn main() {
    let assets = TempAssets::start();

    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    dioxus_desktop::launch_with_props(
        App,
        (),
        Config::new()
            .with_window(WindowBuilder::new().with_title("Mio ◯"))
            .with_close_behaviour(dioxus_desktop::WindowCloseBehaviour::LastWindowExitsApp),
    );

    assets.safe_exit();
}
