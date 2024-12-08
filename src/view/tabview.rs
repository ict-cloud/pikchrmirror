use floem::{prelude::*, ViewId};
use floem::prelude::editor::text::Document;
use std::rc::Rc;
use crate::parser::pikchr::pik_svgstring;

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

fn render_button(i_txt: String, i_svgstr: RwSignal<String>) -> Button {
  // try to find the size of the window
  button("Render").action({
    let ltxt = i_txt.clone();
    move || {
    log::debug!("Render Button clicked");
    let (i, e) = pik_svgstring(&ltxt, i_svgstr.get_untracked().as_str());
    log::warn!("errtext: {}", e);
    i_svgstr.set(i);
  }})
}

fn clear_button(i_doc: &Rc<dyn Document>) -> Button {
  todo!()
}

fn save_button(i_svgstr: RwSignal<String>) -> Button {
  todo!()
}

pub fn tabbar_container(i_doc: &Rc<dyn Document>, i_pikstr: RwSignal<String>, i_preview_id: ViewId) -> impl IntoView {
  let doctxt: String = i_doc.text().into();
  let render = render_button(doctxt, i_pikstr);
  let clear = clear_button(&i_doc);
  let save = save_button(i_pikstr);
  container((
    render,
    clear,
    save
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