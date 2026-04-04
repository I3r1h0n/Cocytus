pub mod theme;
pub mod widgets;

mod app;
mod types;
mod update;
mod views;

// Re-exports
pub use app::App;
pub use types::*;

// Constants
pub(crate) const APP_NAME: &str = "Cocytus";
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const APP_AUTHOR: &str = "I3r1h0n";
pub(crate) const SYSTEM32_PATH: &str = "\\Windows\\System32";
pub(crate) const MAX_VISIBLE_FILES: usize = 500;
pub(crate) const MAX_VISIBLE_SYMBOLS: usize = 1000;

/// GUI entry point
pub fn run() -> iced::Result {
    let icon = iced::window::icon::from_file_data(
        include_bytes!("../../assets/logo.png"),
        None,
    )
    .ok();

    let mut app = iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size((600.0, 640.0))
        .centered()
        .resizable(true);

    if let Some(icon) = icon {
        app = app.window(iced::window::Settings {
            icon: Some(icon),
            ..Default::default()
        });
    }

    app.run_with(App::new)
}

/// Helper
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
