use iced::highlighter;
use iced::widget::text_editor;
use iced::{Element, Task, Theme};

pub mod message;
pub mod model;
pub mod update;
pub mod view;

pub use message::Message;

#[cfg(test)]
pub mod tests;

pub struct MirrorApp {
    pub text: String,
    pub content: text_editor::Content,
    pub svg: String,
    pub error: String,
    pub theme: highlighter::Theme,
}

impl MirrorApp {
    pub fn title(&self) -> String {
        String::from("PikchrMirror")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::update(self, message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::view(&self)
    }

    pub fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
