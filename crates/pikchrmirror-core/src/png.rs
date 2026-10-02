use resvg::{
    tiny_skia::{self, Pixmap},
    usvg,
};

/// Upper bound on either output dimension, to keep a hostile scale from exhausting memory.
const MAX_PNG_DIMENSION: u32 = 16_384;

fn try_pm_from_svgstr(i_svgstr: &str, i_scale: f32) -> Result<Pixmap, String> {
    let tree = {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();

        let safe_svgstr = if i_svgstr.starts_with("<!-- empty pikchr diagram -->") {
            r#"<svg xmlns='http://www.w3.org/2000/svg'/>"#
        } else {
            i_svgstr
        };
        usvg::Tree::from_str(safe_svgstr, &opt).map_err(|e| format!("Invalid SVG: {e}"))?
    };

    let intrinsic = tree.size().to_int_size();
    let scale = if i_scale.is_finite() {
        i_scale.max(0.1)
    } else {
        return Err(format!("Invalid scale: {i_scale}"));
    };
    log::debug!("svg->png scale: {}", scale);

    let transform = tiny_skia::Transform::from_scale(scale, scale);
    let out_w = ((intrinsic.width() as f32) * scale).round().max(1.0);
    let out_h = ((intrinsic.height() as f32) * scale).round().max(1.0);
    if out_w > MAX_PNG_DIMENSION as f32 || out_h > MAX_PNG_DIMENSION as f32 {
        return Err(format!(
            "Output size {out_w}x{out_h} exceeds the {MAX_PNG_DIMENSION}px limit; use a smaller scale"
        ));
    }

    let mut pixmap = tiny_skia::Pixmap::new(out_w as u32, out_h as u32)
        .ok_or_else(|| format!("Cannot allocate a {out_w}x{out_h} pixmap"))?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}

/// Non-panicking variant of [`svg_to_png`]; invalid SVG or sizes are reported as `Err`.
pub fn try_svg_to_png(i_svg: &str, i_scale: Option<f32>) -> Result<Vec<u8>, String> {
    let scale = i_scale.unwrap_or(2.0);
    log::debug!("svg_to_png, scale: {}", scale);
    try_pm_from_svgstr(i_svg, scale)?
        .encode_png()
        .map_err(|e| format!("PNG encoding failed: {e}"))
}

pub fn svg_to_png(i_svg: &str, i_scale: Option<f32>) -> Vec<u8> {
    try_svg_to_png(i_svg, i_scale).expect("Valid SVG rendered to PNG")
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
    fn test_try_svg_to_png_rejects_invalid_svg() {
        assert!(try_svg_to_png("not an svg", None).is_err());
    }

    #[test]
    fn test_try_svg_to_png_rejects_oversized_output() {
        assert!(try_svg_to_png(TST_SVG, Some(1.0e6)).is_err());
        assert!(try_svg_to_png(TST_SVG, Some(f32::NAN)).is_err());
    }

    #[test]
    fn test_try_svg_to_png_handles_empty_pikchr_diagram() {
        let png = try_svg_to_png("<!-- empty pikchr diagram -->\n", Some(1.0)).unwrap();
        assert!(!png.is_empty());
    }

    #[test]
    fn test_scale_affects_output_size() {
        let png_1x = svg_to_png(TST_SVG, Some(1.0));
        let png_4x = svg_to_png(TST_SVG, Some(4.0));
        assert!(png_4x.len() > png_1x.len(), "scaled image should be larger");
    }
}
