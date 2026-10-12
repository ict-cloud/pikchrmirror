use super::{ExportFormat, Message, MirrorApp};
use crate::filehandler::actions;
use crate::img::png;
use iced::widget::text_editor;
use iced::Task;

#[cfg(feature = "llm")]
use iced::window;

#[cfg(feature = "llm")]
use crate::app::chat_state::Role;

pub fn update(model: &mut MirrorApp, message: Message) -> Task<Message> {
    match message {
        Message::TextEditorAction(action) => {
            model.content.perform(action);
            let (svg, error) = pikchrmirror_core::pik_svgstring(&model.content.text(), &model.svg);
            model.svg = svg;
            model.error = error;

            Task::none()
        }
        Message::NewFile => {
            model.content = text_editor::Content::new();
            model.text = String::new();
            let (svg, error) = pikchrmirror_core::pik_svgstring(&model.content.text(), &model.svg);
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
                ExportFormat::Png => Task::perform(
                    png::save_svg_as_png(
                        None,
                        model.svg.clone(),
                        Some(model.export_quality.scale()),
                    ),
                    |_| Message::ImageExported,
                ),
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
                    let (svg, error) = pikchrmirror_core::pik_svgstring(&model.text, "");
                    model.svg = svg;
                    model.error = error;
                }
                Err(actions::Error::DialogClosed) => {}
                Err(error) => {
                    model.file_error = Some(format!("Error opening file: {:?}", error));
                    model.content = text_editor::Content::new();
                    model.text = String::new();
                    let (svg, error) = pikchrmirror_core::pik_svgstring(&model.text, &model.svg);
                    model.svg = svg;
                    model.error = error;
                }
            }
            Task::none()
        }
        Message::ExportQualitySelected(quality) => {
            model.export_quality = quality;
            Task::none()
        }
        Message::AcknowledgeError => {
            model.file_error = None;
            Task::none()
        }
        Message::WindowClosed(id) => {
            if model.main_window == Some(id) {
                // The editor window is the application: closing it also closes the chat.
                return iced::exit();
            }
            #[cfg(feature = "llm")]
            if model.chat.window == Some(id) {
                model.chat.window = None;
            }
            Task::none()
        }
        #[cfg(feature = "llm")]
        Message::ToggleChat => {
            if let Some(id) = model.chat.window.take() {
                return window::close(id);
            }
            let (id, open) = window::open(window::Settings {
                size: iced::Size::new(480.0, 640.0),
                min_size: Some(iced::Size::new(320.0, 320.0)),
                ..window::Settings::default()
            });
            model.chat.window = Some(id);
            let mut tasks = vec![open.discard()];
            // Load lazily on first open; a failed load is retried on the next open.
            if matches!(
                model.chat.status,
                crate::app::chat_state::ModelStatus::Unloaded
                    | crate::app::chat_state::ModelStatus::Error(_)
            ) {
                model.chat.status = crate::app::chat_state::ModelStatus::Loading;
                tasks.push(Task::perform(
                    pikchrmirror_mcp::llm::load_model(),
                    Message::ModelLoaded,
                ));
            }
            Task::batch(tasks)
        }
        #[cfg(feature = "llm")]
        Message::ModelLoaded(Ok(m)) => {
            model.chat.model = Some(m);
            model.chat.status = crate::app::chat_state::ModelStatus::Ready;
            Task::none()
        }
        #[cfg(feature = "llm")]
        Message::ModelLoaded(Err(e)) => {
            model.chat.status = crate::app::chat_state::ModelStatus::Error(e);
            Task::none()
        }
        #[cfg(feature = "llm")]
        Message::ChatInputChanged(s) => {
            model.chat.input = s;
            Task::none()
        }
        #[cfg(feature = "llm")]
        Message::ChatSubmit => {
            let user_msg = model.chat.input.trim().to_string();
            // Keep the typed text when the model is not ready, rather than
            // dropping it and leaving `generating` stuck.
            if user_msg.is_empty() || !model.chat.can_send() {
                return Task::none();
            }
            model.chat.input.clear();
            model.chat.messages.push(crate::app::chat_state::ChatTurn {
                role: Role::User,
                text: user_msg.clone(),
            });
            model.chat.generating = true;

            let Some(model_ref) = model.chat.model.clone() else {
                model.chat.generating = false;
                return Task::none();
            };
            let history = model
                .chat
                .messages
                .iter()
                .map(|turn| {
                    (
                        match turn.role {
                            Role::User => "user".to_string(),
                            Role::Assistant => "assistant".to_string(),
                        },
                        turn.text.clone(),
                    )
                })
                .collect::<Vec<_>>();
            // Same compile-checked, self-repairing generation the MCP tool uses.
            // `model.text` is only the initial/opened text; the editor content is current.
            let current_src = model.content.text();
            Task::perform(
                async move {
                    pikchrmirror_mcp::llm::generate_pikchr(model_ref, history, &current_src).await
                },
                |result| Message::ChatReplyDone(result.unwrap_or_else(|e| format!("Error: {}", e))),
            )
        }
        #[cfg(feature = "llm")]
        Message::ChatReplyDone(reply) => {
            model.chat.messages.push(crate::app::chat_state::ChatTurn {
                role: Role::Assistant,
                text: reply.clone(),
            });
            model.chat.generating = false;
            model.chat.proposed_code = pikchrmirror_mcp::llm::extract_pikchr_block(&reply);
            Task::none()
        }
        #[cfg(feature = "llm")]
        Message::ApplyProposedCode => {
            if let Some(code) = model.chat.proposed_code.take() {
                model.text = code.clone();
                model.content = text_editor::Content::with_text(&code);
                let (svg, error) = pikchrmirror_core::pik_svgstring(&code, "");
                model.svg = svg;
                model.error = error;
            }
            Task::none()
        }
    }
}
