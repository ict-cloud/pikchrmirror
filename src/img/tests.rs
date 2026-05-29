use crate::img::png;

const TST_SVG: &str = r#"
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.2 Tiny//EN"
                     "http://www.w3.org/TR/SVGTiny12/DTD/SVGTiny12.dtd">
<svg width="100%" height="100%" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <rect x="20" y="20" width="60" height="60" fill=" ##FF0000 "/>
</svg>"#;

#[test]
fn test_img_encode() {
    let png = png::svg_to_png(TST_SVG, None);
    assert!(png.len() > 0);
}

#[test]
fn test_scale_affects_output_size() {
    let png_1x = png::svg_to_png(TST_SVG, Some(1.0));
    let png_4x = png::svg_to_png(TST_SVG, Some(4.0));
    assert!(png_4x.len() > png_1x.len());
}

#[tokio::test]
async fn test_save_svg_as_png() {
    let path = std::path::PathBuf::from("./test_async.png");
    let result = png::save_svg_as_png(Some(path.clone()), TST_SVG.to_string(), None).await;
    assert!(result.is_ok());
    assert!(path.exists());
    std::fs::remove_file(path).unwrap();
}
