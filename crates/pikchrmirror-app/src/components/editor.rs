use crate::app::Message;
use iced::widget::{text, text_editor};
use iced::{Element, Fill};

pub fn text_editor_component(
    content: &text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'static,
) -> Element<'_, Message> {
    text_editor(content)
        .height(Fill)
        .wrapping(text::Wrapping::Word)
        .on_action(on_action)
        .into()
}
