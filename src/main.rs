mod app;
mod img;
mod parser;

use app::MirrorApp;

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    iced::application(MirrorApp::title, MirrorApp::update, MirrorApp::view).run()
}
