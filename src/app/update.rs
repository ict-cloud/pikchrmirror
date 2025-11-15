use super::{Message, MirrorApp};
use crate::filehandler::actions;
use crate::parser;
use iced::widget::text_editor;
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
        Message::NewFile => {
            model.content = text_editor::Content::new();
            model.text = String::new();
            let (svg, error) = parser::pikchr::pik_svgstring(&model.content.text(), &model.svg);
            model.svg = svg;
            model.error = error;

            Task::none()
        }
        Message::OpenFile => Task::none(),
        Message::SaveFile => Task::perform(actions::save_file(None, model.svg.clone()), |_| {
            Message::FileSaved
        }),
        Message::ThemeSelected(theme) => {
            model.theme = theme;
            Task::none()
        }
        Message::FileSaved => Task::none(),
    }
}
