mod app;
mod components;
mod filehandler;
mod img;
mod parser;

use app::MirrorApp;
use iced::Font;

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    iced::application(MirrorApp::default, MirrorApp::update, MirrorApp::view)
        .theme(MirrorApp::theme)
        .font(include_bytes!("../fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run()
}
