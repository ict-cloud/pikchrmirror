use resvg::{usvg, tiny_skia};

pub fn svgstr_to_png(i_svgstr: &str, i_file_path: &str ) {
  let tree = {
    let mut opt = usvg::Options::default();

    opt.fontdb_mut().load_system_fonts();

    usvg::Tree::from_str(i_svgstr, &opt).expect("Valid SVG Tree")
  };

  let pixmap_size = tree.size().to_int_size();
  let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).expect("Valid parameters");
  resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
  pixmap.save_png(i_file_path).expect("PNG successfully saved");
}