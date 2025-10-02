use super::{Message, MirrorApp};
use crate::parser;
use iced::Task;

pub fn update(model: &mut MirrorApp, message: Message) -> Task<Message> {
    match message {
        Message::TextInputChanged(text) => {
            model.text = text;
            let (svg, error) = parser::pikchr::pik_svgstring(&model.text, &model.svg);
            model.svg = svg;
            model.error = error;
            Task::none()
        }
        Message::TextEditorAction(action) => {
            model.content.perform(action);
            let (svg, error) = parser::pikchr::pik_svgstring(&model.content.text(), &model.svg);
            model.svg = svg;
            model.error = error;

            Task::none()
        }
    }
}
