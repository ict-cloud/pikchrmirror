use super::icons;
use crate::app::Message;
use iced::highlighter;
use iced::widget::{button, center, container, pick_list, row, toggler, tooltip};
use iced::{Center, Element, Task};

pub fn controls_row() -> Element<'static, Message> {
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
        iced::widget::horizontal_space(),
        // toggler(self.word_wrap)
        //     .label("Word Wrap")
        //     .on_toggle(Message::WordWrapToggled),
        // pick_list(
        //     highlighter::Theme::ALL,
        //     Some(self.theme),
        //     Message::ThemeSelected
        // )
        // .text_size(14)
        // .padding([5, 10])
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center(content).width(30));

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
