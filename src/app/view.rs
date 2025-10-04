use super::{Message, MirrorApp};
use iced::widget::{column, container, row, svg, text, text_editor};
use iced::{Element, Fill, Length};

pub fn view(model: &MirrorApp) -> Element<Message> {
    let editor = text_editor(&model.content)
        .height(Fill)
        .wrapping(text::Wrapping::Word)
        .on_action(Message::TextEditorAction);

    let svg_handle = svg::Handle::from_memory(model.svg.as_bytes().to_vec());
    let svg_display = svg(svg_handle).width(Length::Fill).height(Length::Fill);

    let content = row![
        column![editor].width(Length::FillPortion(1)),
        column![
            container(svg_display)
                .width(Length::Fill)
                .height(Length::Fill),
            text(&model.error)
        ]
        .width(Length::FillPortion(1))
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}
