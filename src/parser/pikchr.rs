use pikchr::{Pikchr, PikchrFlags};

// fn scale_svg(svg_string: &str, scale_factor: f64) -> Result<String, String> {
//     let opt = Options::default();
//     let mut tree = Tree::from_str(svg_string, &opt).map_err(|e| e.to_string())?;

//     // Get the root node
//     let root = tree.root();

//     // Get the current viewBox attribute
//     let viewbox_attr = root.attributes().get("viewBox").ok_or("ViewBox attribute not found")?;
//     let viewbox = viewbox_attr.value().parse::<Vec<f64>>().map_err(|e| e.to_string())?;

//     // Calculate the new viewBox values
//     let new_width = viewbox[2] * scale_factor;
//     let new_height = viewbox[3] * scale_factor;
//     let new_viewbox = format!("{} {} {} {}", viewbox[0], viewbox[1], new_width, new_height);

//     // Set the new viewBox attribute
//     root.set_attribute("viewBox", &new_viewbox);

//     // Serialize the scaled SVG tree back to a string
//     let scaled_svg_string = tree.to_string();

//     Ok(scaled_svg_string)
// }

pub fn pik_svgstring(i_raw: &str, i_svg_old: &str) -> (String, String) {
  let mut pik_err = String::from("");
  let svg_str = match Pikchr::render(i_raw, None, PikchrFlags::default()) {
      Ok(p) => p.rendered().to_owned(),
      Err(e) => {pik_err = e.to_owned(); i_svg_old.to_owned()},
  };

  (svg_str, pik_err)
}