use std::str::Bytes;

use pikchr::{Pikchr, PikchrFlags};

pub fn pik_svgstring(i_raw: &str, i_svg_old: &str) -> (String, String) {
  let mut pik_err = String::from("");
  let svg_str = match Pikchr::render(i_raw, None, PikchrFlags::default()) {
      Ok(p) => p.rendered().to_owned(),
      Err(e) => {pik_err = e.to_owned(); i_svg_old.to_owned()},
  };

  (svg_str, pik_err)
}

pub fn pik_preview_width(i_rawstr: &str, i_width: f64) -> Vec<u8> {
  let pikrendr = Pikchr::render(i_rawstr, None, PikchrFlags::default());
  let error_svg_tmpl = r#"
  <svg xmlns='http://www.w3.org/2000/svg'/>
  <text>{error}</text>
  "#;
  let svgstr = match pikrendr {
    Ok(p) => p.rendered().to_owned(),
    Err(e) => {
      format!(
        r#"
  <svg xmlns='http://www.w3.org/2000/svg'/>
  <text>{}</text>
  "#, e.as_str())
    }
  };
  println!("{}", svgstr);
  svgstr.as_bytes().to_owned()
}