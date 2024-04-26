use dialog::DialogBox;
use iced::{
    window, Application, Settings, Size,
};

mod app;
mod blacklist;
mod detector;
mod paths;

#[tokio::main]
async fn main() {
    // These are essential app initialisation calls. If any of these methods fail,
    // then we cannot reliably run the app, so we exit execution gracefully.
    if let Err(err) = paths::create_app_dir() {
        display_error(&err.to_string());
        return
    }

    if let Err(err) = paths::create_init_file_if_not_exists() {
        display_error(&err.to_string());
        return
    }

    if let Err(err) = paths::create_blacklist_file_if_not_exists() {
        display_error(&err.to_string());
        return
    }

    if let Err(err) = paths::download_rten_models().await {
        display_error(&err.to_string());
        return
    }

    if let Err(err) = paths::download_banner_file().await {
        display_error(&err.to_string());
        return
    }

    let settings: Settings<()> = Settings {
        window: window::Settings {
            size: Size {
                width: 400f32,
                height: 380f32,
            },
            resizable: false,
            decorations: true,
            ..Default::default()
        },
        ..Default::default()
    };

    app::BlitzApp::run(settings).unwrap()
}

/// Displays an error message in a GUI pop-up for an error propogated before
/// initialisation of the main application.
///
/// # Arguments
/// * `error` - The error message to display.
fn display_error(message: &str) {
    dialog::Message::new(message)
        .title(message)
        .show()
        .expect(format!("Could not display the error dialog: {message}").as_str());
}
