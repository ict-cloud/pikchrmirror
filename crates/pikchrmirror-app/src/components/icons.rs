use iced::widget::text;
use iced::{Element, Font};

pub fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E24D}')
}

pub fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0E161}')
}

pub fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E2C8}')
}

pub fn export_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0E169}')
}

pub fn svg_export_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E251}')
}

pub fn png_export_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E3F4}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("Material Icons");

    text(codepoint)
        .font(ICON_FONT)
        .shaping(text::Shaping::Basic)
        .into()
}
