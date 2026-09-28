use resvg::{
    tiny_skia::{self, Pixmap},
    usvg,
};

fn pm_from_svgstr(i_svgstr: &str, i_scale: f32) -> Pixmap {
    let tree = {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();

        let safe_svgstr = if i_svgstr.starts_with("<!-- empty pikchr diagram -->") {
            r#"<svg xmlns='http://www.w3.org/2000/svg'/>"#
        } else {
            i_svgstr
        };
        usvg::Tree::from_str(safe_svgstr, &opt).expect("Valid SVG Tree")
    };

    let intrinsic = tree.size().to_int_size();
    let scale = i_scale.max(0.1);
    log::debug!("svg->png scale: {}", scale);

    let transform = tiny_skia::Transform::from_scale(scale, scale);
    let out_w = ((intrinsic.width() as f32) * scale).round().max(1.0) as u32;
    let out_h = ((intrinsic.height() as f32) * scale).round().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h).expect("Valid pixmap size");
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap
}

pub fn svg_to_png(i_svg: &str, i_scale: Option<f32>) -> Vec<u8> {
    let scale = i_scale.unwrap_or(2.0);
    log::debug!("svg_to_png, scale: {}", scale);
    let pm = pm_from_svgstr(i_svg, scale);
    pm.encode_png().expect("PNG encoded")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TST_SVG: &str = r#"
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.2 Tiny//EN"
                     "http://www.w3.org/TR/SVGTiny12/DTD/SVGTiny12.dtd">
<svg width="100%" height="100%" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <rect x="20" y="20" width="60" height="60" fill=" ##FF0000 "/>
</svg>"#;

    #[test]
    fn test_img_encode() {
        let png = svg_to_png(TST_SVG, None);
        assert!(!png.is_empty());
    }

    #[test]
    fn test_scale_affects_output_size() {
        let png_1x = svg_to_png(TST_SVG, Some(1.0));
        let png_4x = svg_to_png(TST_SVG, Some(4.0));
        assert!(png_4x.len() > png_1x.len(), "scaled image should be larger");
    }
}
