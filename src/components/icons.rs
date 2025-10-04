use crate::app::Message;
use iced::widget::text;
use iced::{Element, Font};

pub fn new_icon<'a, Task>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

pub fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

pub fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint)
        .font(ICON_FONT)
        .shaping(text::Shaping::Basic)
        .into()
}
