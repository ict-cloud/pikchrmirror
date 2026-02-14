use crate::img::png;

const TST_SVG: &str = r#"
<svg width="100%" height="100%" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <rect x="20" y="20" width="60" height="60" fill="#FF0000"/>
</svg>"#;

#[test]
fn test_save_to_pngfile() {
    let path = "./test.png";
    png::save_to_pngfile(TST_SVG, path);
    assert!(std::path::Path::new(path).exists());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn test_img_encode() {
    let png = png::svg_to_png(TST_SVG, None);
    assert!(png.len() > 0);
}

#[tokio::test]
async fn test_save_svg_as_png() {
    let path = std::path::PathBuf::from("./test_async.png");
    let result = png::save_svg_as_png(Some(path.clone()), TST_SVG, None).await;
    assert!(result.is_ok());
    assert!(path.exists());
    std::fs::remove_file(path).unwrap();
}
