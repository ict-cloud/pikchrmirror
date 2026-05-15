use super::{ExportFormat, Message, MirrorApp};
use crate::filehandler::actions;
use crate::img::png;
use crate::parser;
use iced::widget::text_editor;
use iced::Task;

pub fn update(model: &mut MirrorApp, message: Message) -> Task<Message> {
    match message {
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
        Message::OpenFile => Task::perform(actions::open_file(), Message::FileOpened),
        Message::SaveFile => Task::perform(actions::save_file(None, model.content.text()), |_| {
            Message::FileSaved
        }),
        Message::Export => {
            model.export_pending = true;
            Task::none()
        }
        Message::ExportAs(format) => {
            model.export_pending = false;
            match format {
                ExportFormat::Svg => {
                    Task::perform(actions::save_svg_file(None, model.svg.clone()), |_| {
                        Message::ImageExported
                    })
                }
                ExportFormat::Png => {
                    Task::perform(png::save_svg_as_png(None, model.svg.clone(), None), |_| {
                        Message::ImageExported
                    })
                }
            }
        }
        Message::ThemeSelected(theme) => {
            model.theme = theme;
            Task::none()
        }
        Message::FileSaved => Task::none(),
        Message::ImageExported => Task::none(),
        Message::FileOpened(result) => {
            match result {
                Ok((_path, content)) => {
                    model.text = content.clone();
                    model.content = text_editor::Content::with_text(&content);
                    let (svg, error) = parser::pikchr::pik_svgstring(&model.text, "");
                    model.svg = svg;
                    model.error = error;
                }
                Err(actions::Error::DialogClosed) => {}
                Err(error) => {
                    model.file_error = Some(format!("Error opening file: {:?}", error));
                    model.content = text_editor::Content::new();
                    model.text = String::new();
                    let (svg, error) = parser::pikchr::pik_svgstring(&model.text, &model.svg);
                    model.svg = svg;
                    model.error = error;
                }
            }
            Task::none()
        }
        Message::AcknowledgeError => {
            model.file_error = None;
            Task::none()
        }
    }
}
