use resvg::{
    tiny_skia::{self, Pixmap},
    usvg,
};
use std::path::PathBuf;

use crate::filehandler::actions::Error;

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

pub async fn save_svg_as_png(
    path: Option<PathBuf>,
    svg_contents: String,
    scale: Option<f32>,
) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .add_filter("PNG Image", &["png"])
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(std::path::Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    let png_data = svg_to_png(&svg_contents, scale);

    tokio::fs::write(&path, png_data)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}
