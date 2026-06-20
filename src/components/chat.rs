#[cfg(feature = "llm")]
use crate::app::chat_state::{ChatState, Role};
#[cfg(feature = "llm")]
use crate::app::Message;
#[cfg(feature = "llm")]
use iced::widget::{button, column, row, scrollable, space, text, text_input};
#[cfg(feature = "llm")]
use iced::{Element, Length};

#[cfg(feature = "llm")]
pub fn chat_panel(state: &ChatState, _generating: bool) -> Element<'_, Message> {
    let messages_col = scrollable(
        column(
            state
                .messages
                .iter()
                .map(|turn| {
                    let label = match turn.role {
                        Role::User => "You",
                        Role::Assistant => "Assistant",
                    };
                    let msg_text = format!("{}: {}", label, turn.text);
                    text(msg_text).into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(10),
    )
    .height(Length::Fill);

    let status_text = format!("{}", &state.status);

    let input_box = text_input("Type a description…", &state.input)
        .on_input(Message::ChatInputChanged)
        .on_submit(Message::ChatSubmit)
        .width(Length::Fill);

    let send_button = button("Send")
        .on_press(Message::ChatSubmit)
        .width(Length::Shrink);

    let apply_section: Element<'_, Message> = if let Some(_code) = &state.proposed_code {
        button("Apply to editor")
            .on_press(Message::ApplyProposedCode)
            .width(Length::Fill)
            .into()
    } else {
        space().width(Length::Fill).into()
    };

    column![
        text("Chat").size(20),
        messages_col,
        text(status_text),
        row![input_box, send_button].spacing(10),
        apply_section,
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
