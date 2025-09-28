mod img;
mod parser;

use iced::widget::{column, row, text, text_input, container};
use iced::application::{Application, Settings};
use iced::{Command, Element, Theme, Length};

#[cfg(test)]
mod tests;

pub fn main() -> iced::Result {
    MirrorApp::run(Settings::default())
}

struct MirrorApp {
    text: String,
    svg: String,
}

#[derive(Debug, Clone)]
enum Message {
    TextInputChanged(String),
}

impl Application for MirrorApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            MirrorApp {
                text: String::from(""),
                svg: String::from(""),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PikchrMirror")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TextInputChanged(text) => {
                self.text = text;
                // TODO: Generate SVG from text using pikchr and resvg
                self.svg = String::from("<svg width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\" stroke=\"green\" stroke-width=\"4\" fill=\"yellow\" /></svg>");
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_input("Enter Pikchr code", &self.text)
            .on_input(Message::TextInputChanged);

        let svg_display = text(&self.svg); // Replace with actual SVG rendering

        let content = row![
            editor.width(Length::FillPortion(1)),
            container(svg_display)
                .width(Length::FillPortion(1))
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
