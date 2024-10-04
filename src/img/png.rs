use resvg::{usvg, tiny_skia};

pub fn svgstr_to_png(i_svgstr: &str) {
  let tree = {
    let mut opt = usvg::Options::default();

    opt.fontdb_mut().load_system_fonts();

    usvg::Tree::from_str(i_svgstr, &opt).unwrap()
  };

  let pixmap_size = tree.size().to_int_size();
  let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
  resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
  pixmap.save_png("test.png").unwrap();
}