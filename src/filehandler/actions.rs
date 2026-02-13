use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

pub async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

pub async fn open_file() -> Result<(PathBuf, String), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    let path = handle.path().to_owned();
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}
