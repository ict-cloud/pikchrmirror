use floem::prelude::*;
use editor::command::{Command, CommandExecuted};
use editor::core::command::EditCommand;
use editor::text::{default_dark_color, SimpleStyling};
use floem::keyboard::{Key, NamedKey};
use crate::view::tabview;

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
        .style(|s| s.size_full().max_width_pct(50.0))
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

    // let svg_result = dyn_container(
    //     move || piksvgstring.get(),
    //     move |pkchr| svg(pkchr)
    //         .style(|s| s.size_full().flex().set(SvgColor, Brush::Solid(Color::BLACK)))
    //     )
    //     .style(|s| s.size_full());

    //let preview_width = editor.view_style().expect("valid style").get(Width);
    //let preview_width = editor.id().get_content_rect();
    //let preview_width = editor.id().inspect();
    //println!("Preview Width {:?}", preview_width.width());

    // preview png should be rendered behind the button and stored in a rw_signal
    // the save version should then be rendered dedicated with another size

    let svg_preview = dyn_container(
        move || pikpreview.get(),
        move |pv| { let pv_ref = pv.clone(); img(move ||pv_ref.to_vec()).style(|s|s.max_width_pct(100.0))} // scaling needs to be dynamic to adapt the dyn_container
      ).scroll().style(|s| s.max_width_pct(50.0).size_full());

    // let tabs_bar = container((
    //     button("Render").action({
    //         let preview_id = svg_preview.id();
    //         let ldoc = doc.clone();
    //         move || {
    //         log::debug!("Render Button clicked");
    //         log::debug!("Preview With Render: {}", preview_id.get_content_rect().width());
    //         let txt: String = ldoc.text().into();
    //         // calculate the preview here and load the picture in a rw_signal
    //         let b = pik_preview_width(&txt, preview_id.get_content_rect().width());
    //         let (i, e) = pik_svgstring(&txt, piksvgstring.get_untracked().as_str());
    //         log::warn!("errtext: {}", e);
    //         piksvgstring.set(i);
    //         pikpreview.set(b);
    //     }}),
    //     button("Clear").action(move || {
    //         doc.edit_single(
    //             Selection::region(0, doc.text().len()),
    //             "",
    //             EditType::DeleteSelection,
    //         );
    //     }),
    //     button("Save PNG").action(move ||{
    //         log::debug!("Save PNG clicked");
    //         save_as(
    //             FileDialogOptions::new()
    //                 .default_name("pikchr.png")
    //                 .title("Save file"),
    //             move |file_info| {
    //                 if let Some(file) = file_info {
    //                     log::debug!("Save file to: {:?}", file.path);
    //                     png::svgstr_to_pngfile(piksvgstring.get().as_str(), file.path[0].as_os_str().to_str().expect("valid path"));
    //                 }
    //             },
    //         );
            
    //     }),
    //     ))
    // .style(|s| {
    //     s.flex_row()
    //         .width_full()
    //         .height(TABBAR_HEIGHT)
    //         .row_gap(5)
    //         .padding(CONTENT_PADDING)
    //         .border_bottom(1)
    //         .border_color(Color::rgb8(205, 205, 205))
    // });
    // doc needs to be dynamic to handover to the function otherwise it will not react on changes.
    let ref_doc = &doc.clone();
    let tabs_bar = tabview::tabbar_container(&ref_doc, &rawdocstr, &pikpreview, svg_preview.id());

    // should be a dyn stack to adjust or react to the new value
    let piked = stack((
        editor,
        svg_preview,
      ))
      .style(|s| s.flex_row().width_full().items_center().justify_center());

    let id = piked.id();
    let inspector = button("Open Inspector")
        .action(move || id.inspect())
        .style(|s| s);

    let view = stack((
        piked,
        tabs_bar,
        inspector,
      ))
      .style(|s| s.size_full().flex_col().items_center().justify_center());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), |m| m.is_empty(), move |_| {
        id.inspect()
      })

}