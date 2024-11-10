use floem::{
  action::save_as, file::FileDialogOptions, keyboard::{Key, NamedKey}, peniko::{Brush, Color}, reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate}, unit::UnitExt, views::{
      button, container, dyn_container, editor::{
          command::{Command, CommandExecuted}, core::{command::EditCommand, editor::EditType, selection::Selection}, text::{default_dark_color, SimpleStyling}
      }, img, stack, svg, text_editor, Decorators, HorizPosition, ObjectFit, SvgColor, VertPosition
  }, IntoView, View
};
use crate::img::png;
use crate::parser::pikchr::pik_svgstring;

const DFLT_TEXT: &str = r#"arrow right 200% "Markdown" "Source"
box rad 10px "Markdown" "Formatter" "(markdown.c)" fit
arrow right 200% "HTML+SVG" "Output"
arrow <-> down 70% from last box.s
box same "Pikchr" "Formatter" "(pikchr.c)" fit
"#;

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

pub fn app_view() -> impl IntoView {

    let (s, e) = pik_svgstring(DFLT_TEXT, "");
    log::debug!("Initial render error: {}", e);
    let piksvgstring = create_rw_signal(s);

    let hide_gutter_a = RwSignal::new(false);

    let editor = text_editor(DFLT_TEXT)
        .styling(SimpleStyling::new())
        .style(|s| s.size_full())
        .editor_style(default_dark_color)
        .editor_style(move |s| s.hide_gutter(hide_gutter_a.get()))
        .pre_command(|ev| {
            if matches!(ev.cmd, Command::Edit(EditCommand::Undo)) {
                println!("Undo command executed on editor B, ignoring!");
                return CommandExecuted::Yes;
            }
            CommandExecuted::No
        })
        .update(move |_dlta| {
            log::debug!("Editor changed");
            // let txt = dlta.editor.unwrap().text().clone();
            // let rawtext = txt.to_string();
            // println!("{:?}", rawtext);
            // let (i, e) = pik_svgstring(&rawtext, piksvgstring.get().as_str());
            // println!("errtext: {}", e);
            // piksvgstring.set(i);
        })
        .placeholder("Some placeholder text");
    let doc = editor.doc();

    // let svg_result = dyn_container(
    //     move || piksvgstring.get(),
    //     move |pkchr| svg(pkchr)
    //         .style(|s| s.size_full().flex().set(SvgColor, Brush::Solid(Color::BLACK)))
    //     )
    //     .style(|s| s.size_full());

    let svg_preview = dyn_container(
        move || piksvgstring.get(),
        move |pkchr| img(move ||png::svg_to_png(&pkchr))
          .style(|s| s.width(320.px()).height(160.px())) // scaling needs to be dynamic to adapt teh dyn_container
    );

    let tabs_bar = container((
        button("Render").action({
            let ldoc = doc.clone();
            move || {
            log::debug!("Render Button clicked");
            let txt: String = ldoc.text().into();
            let (i, e) = pik_svgstring(&txt, piksvgstring.get_untracked().as_str());
            log::warn!("errtext: {}", e);
            piksvgstring.set(i);
        }}),
        button("Clear").action(move || {
            doc.edit_single(
                Selection::region(0, doc.text().len()),
                "",
                EditType::DeleteSelection,
            );
        }),
        button("Save PNG").action(move ||{
            log::debug!("Save PNG clicked");
            save_as(
                FileDialogOptions::new()
                    .default_name("pikchr.png")
                    .title("Save file"),
                move |file_info| {
                    if let Some(file) = file_info {
                        log::debug!("Save file to: {:?}", file.path);
                        png::svgstr_to_pngfile(piksvgstring.get().as_str(), file.path[0].as_os_str().to_str().expect("valid path"));
                    }
                },
            );
            
        }),
        ))
    .style(|s| {
        s.flex_row()
            .width_full()
            .height(TABBAR_HEIGHT)
            .row_gap(5)
            .padding(CONTENT_PADDING)
            .border_bottom(1)
            .border_color(Color::rgb8(205, 205, 205))
    });

    // should be a dyn stack to adjust or react to the new value
    let piked = stack((
        editor,
        svg_preview,
    ))
    .style(|s| s.height_full().width_full().flex_row().items_center().justify_center());

    let view = stack((
        piked,
        tabs_bar,
    ))
    .style(|s| s.size_full().flex_col().items_center().justify_center());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), |m| m.is_empty(), move |_| {
        id.inspect()
    })
}