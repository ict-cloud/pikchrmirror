mod img;
mod parser;

use iced::widget::{column, container, row, svg, text, text_input};
use iced::{Element, Length, Task};

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    iced::run("Pikchr Mirror", MirrorApp::update, MirrorApp::view)
}

#[derive(Default)]
struct MirrorApp {
    text: String,
    svg: String,
    error: String,
}

#[derive(Debug, Clone)]
enum Message {
    TextInputChanged(String),
}

impl MirrorApp {
    fn new(&self) -> (Self, Task<Message>) {
        let initial_text = r##"arrow right 200% "Markdown" "HTML" box rad 10px "Markdown" "(markdown.c)" fit
arrow right 200% "HTML" "pikchr" box rad 10px "HTML" "(pikchr.c)" fit
arrow right 200% "pikchr" "SVG" box rad 10px "pikchr" "(cgi/pikchr.c)" fit
"##;
        let (svg, error) = parser::pikchr::pik_svgstring(initial_text, "");

        (
            Self {
                text: String::from(initial_text),
                svg,
                error,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PikchrMirror")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TextInputChanged(text) => {
                self.text = text;
                let (svg, error) = parser::pikchr::pik_svgstring(&self.text, &self.svg);
                self.svg = svg;
                self.error = error;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let editor =
            text_input("Enter Pikchr code", &self.text).on_input(Message::TextInputChanged);

        let svg_handle = svg::Handle::from_memory(self.svg.as_bytes().to_vec());
        let svg_display = svg(svg_handle).width(Length::Fill).height(Length::Fill);

        let content = row![
            column![editor].width(Length::FillPortion(1)),
            column![
                container(svg_display)
                    .width(Length::Fill)
                    .height(Length::Fill),
                text(&self.error)
            ]
            .width(Length::FillPortion(1))
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}
