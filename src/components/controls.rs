use super::icons;
use crate::app::{ExportFormat, ExportQuality, Message};
use iced::highlighter;
use iced::widget::{button, center, container, pick_list, row, space, text, tooltip};
use iced::{Background, Border, Center, Color, Element, Length, Shadow};

pub fn controls_row(
    selected_theme: highlighter::Theme,
    export_pending: bool,
    export_quality: ExportQuality,
) -> Element<'static, Message> {
    let export_controls: Element<'static, Message> = if export_pending {
        row![
            action_labeled(
                icons::svg_export_icon::<Message>(),
                "SVG",
                "Export as SVG",
                Some(Message::ExportAs(ExportFormat::Svg)),
            ),
            action_labeled(
                icons::png_export_icon::<Message>(),
                "PNG",
                "Export as PNG",
                Some(Message::ExportAs(ExportFormat::Png)),
            ),
            pick_list(
                ExportQuality::ALL,
                Some(export_quality),
                Message::ExportQualitySelected,
            )
            .text_size(12)
            .padding([4, 8]),
        ]
        .spacing(4)
        .into()
    } else {
        action(
            icons::export_icon::<Message>(),
            "Export",
            Some(Message::Export),
        )
    };

    #[cfg(feature = "llm")]
    {
        row![
            action(
                icons::new_icon::<Message>(),
                "New file",
                Some(Message::NewFile)
            ),
            action(
                icons::open_icon::<Message>(),
                "Open file",
                Some(Message::OpenFile)
            ),
            action(
                icons::save_icon::<Message>(),
                "Save file",
                Some(Message::SaveFile)
            ),
            export_controls,
            action(
                text("Chat"),
                "Toggle chat panel",
                Some(Message::ToggleChat)
            ),
            space().width(Length::Fill),
            pick_list(
                highlighter::Theme::ALL,
                Some(selected_theme),
                Message::ThemeSelected
            )
            .text_size(12)
            .padding([4, 8])
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    }

    #[cfg(not(feature = "llm"))]
    {
        row![
            action(
                icons::new_icon::<Message>(),
                "New file",
                Some(Message::NewFile)
            ),
            action(
                icons::open_icon::<Message>(),
                "Open file",
                Some(Message::OpenFile)
            ),
            action(
                icons::save_icon::<Message>(),
                "Save file",
                Some(Message::SaveFile)
            ),
            export_controls,
            space().width(Length::Fill),
            pick_list(
                highlighter::Theme::ALL,
                Some(selected_theme),
                Message::ThemeSelected
            )
            .text_size(12)
            .padding([4, 8])
        ]
        .spacing(10)
        .align_y(Center)
        .into()
    }
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center(content).width(30).height(30));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

fn action_labeled<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    format_label: &'a str,
    tooltip_label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(
        row![
            center(content).width(30).height(30),
            text(format_label).size(12)
        ]
        .align_y(Center)
        .spacing(2),
    )
    .padding([0, 6])
    .style(export_format_style);

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            tooltip_label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

fn export_format_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.0, 0.69, 0.63),
        button::Status::Pressed => Color::from_rgb(0.0, 0.45, 0.40),
        _ => Color::from_rgb(0.0, 0.59, 0.53),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}
