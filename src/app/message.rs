use iced::{highlighter, widget::text_editor::Action};

#[derive(Debug, Clone)]
pub enum Message {
    TextEditorAction(Action),
    NewFile,
    OpenFile,
    SaveFile,
    Export,
    ExportAs(crate::app::ExportFormat),
    ThemeSelected(highlighter::Theme),
    FileSaved,
    ImageExported,
    FileOpened(Result<(std::path::PathBuf, String), crate::filehandler::actions::Error>),
    ExportQualitySelected(crate::app::ExportQuality),
    AcknowledgeError,
}
