use std::path::PathBuf;

use crate::filehandler::actions::Error;

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

    let png_data = pikchrmirror_core::svg_to_png(&svg_contents, scale);

    tokio::fs::write(&path, png_data)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}
