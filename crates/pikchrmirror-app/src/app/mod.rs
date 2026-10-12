use iced::highlighter;
use iced::widget::text_editor;
use iced::{window, Element, Subscription, Task, Theme};

pub mod message;
pub mod model;
pub mod update;
pub mod view;

#[cfg(feature = "llm")]
pub mod chat_state;

pub use message::Message;

#[cfg(feature = "llm")]
pub use chat_state::ChatState;

#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Svg,
    Png,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Svg => write!(f, "SVG"),
            ExportFormat::Png => write!(f, "PNG"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportQuality {
    Standard,
    High,
    Ultra,
}

impl ExportQuality {
    pub const ALL: &'static [ExportQuality] = &[
        ExportQuality::Standard,
        ExportQuality::High,
        ExportQuality::Ultra,
    ];

    pub fn scale(self) -> f32 {
        match self {
            ExportQuality::Standard => 1.0,
            ExportQuality::High => 2.0,
            ExportQuality::Ultra => 4.0,
        }
    }
}

impl std::fmt::Display for ExportQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportQuality::Standard => write!(f, "1x"),
            ExportQuality::High => write!(f, "2x"),
            ExportQuality::Ultra => write!(f, "4x"),
        }
    }
}

pub struct MirrorApp {
    /// Main editor window; closing it quits the application.
    pub main_window: Option<window::Id>,
    pub text: String,
    pub content: text_editor::Content,
    pub svg: String,
    pub error: String,
    pub theme: highlighter::Theme,
    pub file_error: Option<String>,
    pub export_pending: bool,
    pub export_quality: ExportQuality,
    #[cfg(feature = "llm")]
    pub chat: ChatState,
}

impl MirrorApp {
    /// Initial state plus the task that opens the main window.
    pub fn boot() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());
        let app = Self {
            main_window: Some(id),
            ..Self::default()
        };
        (app, open.discard())
    }

    #[cfg_attr(not(feature = "llm"), allow(unused_variables))]
    pub fn title(&self, window: window::Id) -> String {
        #[cfg(feature = "llm")]
        if self.chat.window == Some(window) {
            return String::from("PikchrMirror – Chat");
        }
        String::from("PikchrMirror")
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::update(self, message)
    }

    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        view::view(self, window)
    }

    pub fn theme(&self, _window: window::Id) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
