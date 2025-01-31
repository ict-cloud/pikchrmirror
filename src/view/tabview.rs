use crate::img::png;
use crate::parser::pikchr::*;
use editor::core::editor::EditType;
use editor::core::selection::Selection;
use floem::action::save_as;
use floem::file::FileDialogOptions;
use floem::prelude::editor::text::Document;
use floem::{prelude::*, ViewId};
use std::rc::Rc;

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

fn render_button(
    i_rawstr: &RwSignal<String>,
    i_pngprev: &RwSignal<Vec<u8>>,
    i_preview_id: ViewId,
) -> Button {
    button("Render").action({
        let lpreview_id = i_preview_id;
        let lpngprev = *i_pngprev;
        let l_svgstr = *i_rawstr;
        move || {
            log::debug!("Render Button clicked");
            let b = pik_preview_width(
                l_svgstr.get_untracked().as_str(),
                lpreview_id.get_content_rect().width(),
            );
            lpngprev.set(b);
        }
    })
}

fn clear_button(i_doc: &Rc<dyn Document>) -> Button {
    let doc_clone = i_doc.clone();
    button("Clear").action(move || {
        doc_clone.edit_single(
            Selection::region(0, doc_clone.text().len()),
            "",
            EditType::DeleteSelection,
        );
    })
}

fn save_button(i_rawsvgstr: RwSignal<String>) -> Button {
    button("Save PNG").action(move || {
        log::debug!("Save PNG clicked");
        let (svgstr, _) = pik_svgstring(i_rawsvgstr.get_untracked().as_str(), "");
        save_as(
            FileDialogOptions::new()
                .default_name("pikchr.png")
                .title("Save file"),
            move |file_info| {
                if let Some(file) = file_info {
                    log::debug!("Save file to: {:?}", file.path);
                    png::svgstr_to_pngfile(
                        svgstr.as_str(),
                        file.path[0].as_os_str().to_str().expect("valid path"),
                    );
                }
            },
        );
    })
}

pub fn tabbar_container(
    i_doc: &Rc<dyn Document>,
    i_rawstr: &RwSignal<String>,
    i_pngpreview: &RwSignal<Vec<u8>>,
    i_preview_id: ViewId,
) -> impl IntoView {
    let render = render_button(i_rawstr, i_pngpreview, i_preview_id);
    let clear = clear_button(i_doc);
    let save = save_button(*i_rawstr);
    container((render, clear, save)).style(|s| {
        s.flex_row()
            .width_full()
            .height(TABBAR_HEIGHT)
            .row_gap(5)
            .padding(CONTENT_PADDING)
            .border_bottom(1)
            .border_color(Color::from_rgb8(205, 205, 205))
    })
}
