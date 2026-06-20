mod app;
mod components;
mod filehandler;
mod img;
#[cfg(feature = "llm")]
mod llm;
mod parser;

use app::MirrorApp;
use iced::Font;

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    iced::application(MirrorApp::default, MirrorApp::update, MirrorApp::view)
        .theme(MirrorApp::theme)
        .font(include_bytes!("../fonts/material-design-icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .title(MirrorApp::title)
        .run()
}
