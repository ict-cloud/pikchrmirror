use crate::view::tabview;
use editor::command::{Command, CommandExecuted};
use editor::core::command::EditCommand;
use editor::text::{default_dark_color, SimpleStyling};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;

const DFLT_TEXT: &str = r#"arrow right 200% "Markdown" "Source"
box rad 10px "Markdown" "Formatter" "(markdown.c)" fit
arrow right 200% "HTML+SVG" "Output"
arrow <-> down 70% from last box.s
box same "Pikchr" "Formatter" "(pikchr.c)" fit
"#;

pub fn app_view() -> impl IntoView {
    let rawdocstr = create_rw_signal(DFLT_TEXT.to_string());

    let p: Vec<u8> = Vec::new();

    let pikpreview = create_rw_signal(p);

    let hide_gutter_a = RwSignal::new(false);

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
    let doc = editor.doc();

    // preview png should be rendered behind the button and stored in a rw_signal
    // the save version should then be rendered dedicated with another size

    let svg_preview = dyn_container(
        move || pikpreview.get(),
        move |pv| {
            let pv_ref = pv.clone();
            img(move || pv_ref.to_vec())
        }, // scaling needs to be dynamic to adapt the dyn_container
    )
    .scroll()
    .style(|s| s.flex_col().size_full());

    // doc needs to be dynamic to handover to the function otherwise it will not react on changes.
    let ref_doc = &doc.clone();
    let tabs_bar = tabview::tabbar_container(ref_doc, &rawdocstr, &pikpreview, svg_preview.id());

    // should be a dyn stack to adjust or react to the new value
    let piked = stack((editor, svg_preview))
        .style(|s| s.flex_row().size_full().items_center().justify_center());

    let view = stack((piked, tabs_bar))
        .style(|s| s.size_full().flex_col().items_center().justify_center());

    let id = view.id();
    view.on_key_up(
        Key::Named(NamedKey::F11),
        |m| m.is_empty(),
        move |_| id.inspect(),
    )
}
