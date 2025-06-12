use floem::{event::EventPropagation, prelude::*, style::CursorStyle, text::Weight};

const TABBAR_HEIGHT: f64 = 37.0;
const CONTENT_PADDING: f64 = 10.0;

pub fn navbar() -> impl View {
  h_stack((
    tab_button(String::from("Home")),
    tab_button(String::from("Settings")),
    tab_button(String::from("About"))
  ))
  .style(|s| {
    s.flex_row()
        .width_full()
        .height(TABBAR_HEIGHT)
        .col_gap(5)
        .padding(CONTENT_PADDING)
        .border_bottom(1)
        .border_color(Color::from_rgb8(205, 205, 205))
  })
}

fn tab_button(
  this_tab: String
) -> impl IntoView {

  let this_tab2 = this_tab.clone();
  v_stack((
  label(move || this_tab.clone())
      .keyboard_navigable()
      .style(move |s| {
          s.width(70)
              .hover(|s| s.font_weight(Weight::BOLD).cursor(CursorStyle::Pointer))
      }),
    ))
      .on_click(move  |_| {
          println!("Tab clicked: {}", this_tab2);
          EventPropagation::Continue
      })
}