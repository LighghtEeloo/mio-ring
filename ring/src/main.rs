fn main() {
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    dioxus_desktop::launch_with_props(mio_ring::App, (), dioxus_desktop::Config::default());
}
