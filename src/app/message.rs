use iced::{highlighter, widget::text_editor::Action};

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
    TextEditorAction(Action),
    NewFile,
    OpenFile,
    SaveFile,
    ThemeSelected(highlighter::Theme),
    FileSaved,
    FileOpened(Result<(std::path::PathBuf, String), crate::filehandler::actions::Error>),
    AcknowledgeError,
}
