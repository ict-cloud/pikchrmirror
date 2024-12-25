use floem::{prelude::*, ViewId};
use floem::prelude::editor::text::Document;
use editor::core::editor::EditType;
use editor::core::selection::Selection;
use std::rc::Rc;
use crate::parser::pikchr::*;

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

fn render_button(
  i_rawstr: &RwSignal<String>,
  i_pngprev: &RwSignal<Vec<u8>>,
  i_preview_id: ViewId
) -> Button {
  button("Render").action({
    let lpreview_id = i_preview_id.clone();
    let lpngprev = i_pngprev.clone();
    let l_svgstr = i_rawstr.clone();
    move || {
    log::debug!("Render Button clicked");
    let b = pik_preview_width(l_svgstr.get_untracked().as_str(), lpreview_id.get_content_rect().width());
    //let (_, e) = pik_svgstring(&ltxt, l_svgstr.get_untracked().as_str());
    //log::warn!("errtext: {}", e);
    lpngprev.set(b);
  }})
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

fn save_button(i_svgstr: RwSignal<String>) -> Button {
  todo!()
}

pub fn tabbar_container(
  i_doc: &Rc<dyn Document>, 
  i_rawstr: &RwSignal<String>,
  i_pngpreview: &RwSignal<Vec<u8>>, 
  i_preview_id: ViewId
) -> impl IntoView {
  let render = render_button(i_rawstr, i_pngpreview, i_preview_id);
  let clear = clear_button(i_doc);
  //let save = save_button(i_pikstr);
  container((
    render,
    clear,
  //  save
  )).style(|s| {
    s.flex_row()
        .width_full()
        .height(TABBAR_HEIGHT)
        .row_gap(5)
        .padding(CONTENT_PADDING)
        .border_bottom(1)
        .border_color(Color::rgb8(205, 205, 205))
  })

}