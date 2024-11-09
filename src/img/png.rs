use resvg::{tiny_skia::{self, Pixmap}, usvg};

fn pm_from_svgstr(i_svgstr: &str) -> Pixmap {
  let tree = {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    usvg::Tree::from_str(i_svgstr, &opt).expect("Valid SVG Tree")
  };

  let pixmap_size = tree.size().to_int_size();
  // pixmap size should fit a given width of 1600
  // for this the factor needs to be applied to height as well
  //let tree2 = tree.size().scale_to(pixmap_size);
  let scale_factor = 1600.0 / pixmap_size.width() as f32;
  let transform = resvg::tiny_skia::Transform::from_scale(scale_factor, scale_factor);
  let scaled_size = pixmap_size.scale_to_width(1600).expect("successful scaled the size");

  let mut pixmap = tiny_skia::Pixmap::new(scaled_size.width(), scaled_size.height()).expect("Valid parameters");
  resvg::render(&tree, transform, &mut pixmap.as_mut());
  pixmap
}

pub fn svg_to_png(i_svg: &str) -> Vec<u8> {
  let pm = pm_from_svgstr(i_svg);
  pm.encode_png().expect("PNG encoded")
}

pub fn svgstr_to_pngfile(i_svgstr: &str, i_file_path: &str ) {
  let pixmap = pm_from_svgstr(i_svgstr);
  pixmap.save_png(i_file_path).expect("PNG successfully saved");
}