mod app;
mod components;
mod filehandler;
mod img;

use app::MirrorApp;
use iced::Font;

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    // A daemon (instead of a plain application) because the chat assistant lives in
    // its own window: windows are opened and closed explicitly by `MirrorApp`.
    iced::daemon(MirrorApp::boot, MirrorApp::update, MirrorApp::view)
        .subscription(MirrorApp::subscription)
        .theme(MirrorApp::theme)
        .font(include_bytes!("../fonts/material-design-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .title(MirrorApp::title)
        .run()
}
