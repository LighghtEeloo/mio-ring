use mio_gui::App;

fn main() {
    // launch the dioxus app in a webview
    dioxus_desktop::launch(App::run);
}

