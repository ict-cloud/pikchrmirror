use super::MirrorApp;
use crate::parser::pikchr;

use iced::highlighter;
use iced::widget::text_editor;

impl Default for MirrorApp {
    fn default() -> Self {
        let initial_text = r##"arrow right 200% "Markdown" "HTML"
box rad 10px "Markdown" "(markdown.c)" fit
arrow right 200% "HTML" "pikchr"
box rad 10px "HTML" "(pikchr.c)" fit
arrow right 200% "pikchr" "SVG"
box rad 10px "pikchr" "(cgi/pikchr.c)" fit
"##;
        let (svg, error) = pikchr::pik_svgstring(initial_text, "");
        Self {
            text: String::from(initial_text),
            content: text_editor::Content::with_text(initial_text),
            svg,
            error,
            theme: highlighter::Theme::InspiredGitHub,
            file_error: None,
            export_pending: false,
            export_quality: crate::app::ExportQuality::High,
        }
    }
}
