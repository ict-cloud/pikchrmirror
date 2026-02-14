use iced::{highlighter, widget::text_editor::Action};

#[derive(Debug, Clone)]
pub enum Message {
    TextEditorAction(Action),
    NewFile,
    OpenFile,
    SaveFile,
    ExportPNG,
    ThemeSelected(highlighter::Theme),
    FileSaved,
    ImageExported,
    FileOpened(Result<(std::path::PathBuf, String), crate::filehandler::actions::Error>),
    AcknowledgeError,
}
