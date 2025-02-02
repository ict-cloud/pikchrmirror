use editor::command::{Command, CommandExecuted};
use editor::core::command::EditCommand;
use editor::text::{default_dark_color, SimpleStyling};
use floem::prelude::*;

pub const DFLT_TEXT: &str = r#"arrow right 200% "Markdown" "Source"
box rad 10px "Markdown" "Formatter" "(markdown.c)" fit
arrow right 200% "HTML+SVG" "Output"
arrow <-> down 70% from last box.s
box same "Pikchr" "Formatter" "(pikchr.c)" fit
"#;

pub fn textedit_view(i_editorstring: &RwSignal<String>) -> TextEditor {
    let hide_gutter_a = RwSignal::new(false);
    let rawdocstr = *i_editorstring;

    let editor = text_editor(DFLT_TEXT)
        .styling(SimpleStyling::new())
        .style(|s| s.flex_col().size_full())
        .editor_style(default_dark_color)
        .editor_style(move |s| s.hide_gutter(hide_gutter_a.get()))
        .pre_command(|ev| {
            if matches!(ev.cmd, Command::Edit(EditCommand::Undo)) {
                println!("Undo command executed on editor B, ignoring!");
                return CommandExecuted::Yes;
            }
            CommandExecuted::No
        })
        .update(move |dlta| {
            let txt = dlta.deltas().last().unwrap();
            log::debug!("Editor changed \n new delta: {:?}", txt);
            let rawdoc = if txt.new_document_len() == 0 {
                String::from("")
            } else {
                String::from(dlta.editor.unwrap().text().clone())
            };
            rawdocstr.set(rawdoc);
        })
        .placeholder("Some placeholder text");
    editor
}
