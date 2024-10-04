mod img;

use floem::{
    context::{self, UpdateCx}, keyboard::{Key, Modifiers, NamedKey}, peniko::Color, reactive::{create_rw_signal, use_context, RwSignal, SignalGet, SignalUpdate}, views::{
        button, container, dyn_container, editor::{
            self, command::{Command, CommandExecuted}, core::{command::EditCommand, editor::EditType, selection::Selection}, text::{default_dark_color, SimpleStyling}
        }, stack, svg, text_editor, Decorators
    }, IntoView, View
};
use pikchr::{Pikchr, PikchrFlags};
use img::png::{self, svgstr_to_png};

const DFLT_TEXT: &str = r#"arrow right 200% "Markdown" "Source"
box rad 10px "Markdown" "Formatter" "(markdown.c)" fit
arrow right 200% "HTML+SVG" "Output"
arrow <-> down 70% from last box.s
box same "Pikchr" "Formatter" "(pikchr.c)" fit
"#;

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

fn pik_rendered_failsafe(i_pikraw: &str, i_pikinst: &mut Pikchr) -> String {
    let last_rendered = i_pikinst.to_owned();
    let mut render_error = String::from("");
    match Pikchr::render(i_pikraw, None, PikchrFlags::default()) {
        Ok(p) => *i_pikinst = p,
        Err(e) => render_error = e.to_owned(),
    };
    render_error
}

fn pik_svgstring(i_raw: &str, i_svg_old: &str) -> (String, String) {
    let mut pik_err = String::from("");
    let mut svg_str = String::from("");
    match Pikchr::render(i_raw, None, PikchrFlags::default()) {
        Ok(p) => svg_str = p.rendered().to_owned(),
        Err(e) => {pik_err = e.to_owned(); svg_str = i_svg_old.to_owned();},
    };

    (svg_str, pik_err)
}

fn app_view() -> impl IntoView {

    let (s, e) = pik_svgstring(DFLT_TEXT, "");
    println!("Initial render error: {}", e);
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
        .update(move |dlta| {
            println!("Editor changed");
            // let txt = dlta.editor.unwrap().text().clone();
            // let rawtext = txt.to_string();
            // println!("{:?}", rawtext);
            // let (i, e) = pik_svgstring(&rawtext, piksvgstring.get().as_str());
            // println!("errtext: {}", e);
            // piksvgstring.set(i);
        })
        .placeholder("Some placeholder text");
    let doc = editor.doc();

    let svg_result = dyn_container(
        move || svg(piksvgstring.get()).style(|s| s.size_full().flex()),
        move |v| v
        )
        .style(|s| s.size_full());

    let tabs_bar = container((
        button("Render").action({
            let ldoc = doc.clone();
            move || {
            println!("Render Button clicked");
            let txt: String = ldoc.text().into();
            let (i, e) = pik_svgstring(&txt, piksvgstring.get().as_str());
            println!("errtext: {}", e);
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
            println!("Save PNG clicked");
            svgstr_to_png(piksvgstring.get().as_str());
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

    let piked = stack((
        editor,
        svg_result,
    ))
    .style(|s| s.height_full().width_full().flex_row().items_center().justify_center());

    let view = stack((
        piked,
        tabs_bar,
    ))
    .style(|s| s.size_full().flex_col().items_center().justify_center());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| {
        id.inspect()
    })
}

fn main() {
    floem::launch(app_view)
}