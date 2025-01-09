use crate::img::png;

const TST_SVG: &str = r#"
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.2 Tiny//EN"
                     "http://www.w3.org/TR/SVGTiny12/DTD/SVGTiny12.dtd">
<svg width="100%" height="100%" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <rect x="20" y="20" width="60" height="60" fill=" ##FF0000 "/>
</svg>"#;

pub fn it_img_xport() {
  png::svgstr_to_pngfile(TST_SVG, "./test.png");
}

pub fn it_img_encode() {
  let png = png::svg_to_png(TST_SVG, None);
  assert!(png.len() > 0);
}