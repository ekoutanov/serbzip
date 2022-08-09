use std::fs::{create_dir_all, File};
use std::io;
use std::path::Path;

pub enum DownloadToFileError {
    IoError(io::Error),
    HttpError(reqwest::Error)
}

impl From<io::Error> for DownloadToFileError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<reqwest::Error> for DownloadToFileError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

fn create_parent_dirs(path: &impl AsRef<Path>) -> Result<(), io::Error> {
    let path = path.as_ref();
    create_dir_all(path.parent().unwrap())?;
    Ok(())
}

pub fn download_to_file(url: &str, path: impl AsRef<Path>) -> Result<(), DownloadToFileError> {
    let resp = reqwest::blocking::get(url)?;
    let body = resp.bytes()?;
    let mut body_bytes = &body[..];
    create_parent_dirs(&path)?;
    let mut out = File::create(path)?;
    io::copy(&mut body_bytes, &mut out)?;
    Ok(())
}