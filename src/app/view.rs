use super::{Message, MirrorApp};
use crate::components::controls::controls_row;
use crate::components::editor::text_editor_component;
use iced::widget::{button, column, container, row, svg, text};
use iced::{Element, Length};

#[cfg(feature = "llm")]
use crate::components::chat::chat_panel;

pub fn view(model: &MirrorApp) -> Element<'_, Message> {
    if let Some(error) = &model.file_error {
        return container(
            column![
                text("An error occurred:").size(20),
                text(error),
                button("OK").on_press(Message::AcknowledgeError)
            ]
            .spacing(20)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let editor = text_editor_component(&model.content, Message::TextEditorAction);

    let svg_handle = svg::Handle::from_memory(model.svg.as_bytes().to_vec());
    let svg_display = svg(svg_handle).width(Length::Fill).height(Length::Fill);

    let controls = container(controls_row(
        model.theme,
        model.export_pending,
        model.export_quality,
    ))
    .height(Length::Fixed(50.0));

    #[cfg(feature = "llm")]
    let chat_col: Element<'_, Message> = if model.chat.visible {
        column![chat_panel(&model.chat, model.chat.generating)]
            .width(Length::FillPortion(1))
            .into()
    } else {
        column![].width(Length::Shrink).into()
    };

    #[cfg(not(feature = "llm"))]
    let chat_col: Element<'_, Message> = column![].width(Length::Shrink).into();

    let content = column![
        controls,
        row![
            column![editor].width(Length::FillPortion(2)),
            column![
                container(svg_display)
                    .width(Length::Fill)
                    .height(Length::Fill),
                text(&model.error)
            ]
            .width(Length::FillPortion(2)),
            chat_col,
        ]
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
