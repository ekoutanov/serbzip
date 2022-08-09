use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use bincode::error::{DecodeError, EncodeError};
use serbzip::succinct::CowStr;
use serbzip::transcoder::TranscodeError;
use crate::cli::downloader::DownloadToFileError;

#[derive(Debug)]
pub enum CliErrorKind {
    UnsupportedDictionaryFormat,
    UnsupportedBinaryDictionaryFormat,
    InvalidMode,
    NoSuchInputFile,
    NoHomeDir,
    NoSuchDictFile,
}

pub type CliErrorDetail = CowStr;

#[derive(Debug)]
pub struct CliError(pub CliErrorKind, pub CliErrorDetail);

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.0, self.1)
    }
}

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    DictDownloadError(Box<dyn Error>),
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    CliError(CliError),
    TranscodeError(TranscodeError<Box<dyn Error>>),
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<CliError> for AppError {
    fn from(error: CliError) -> Self {
        Self::CliError(error)
    }
}

impl From<EncodeError> for AppError {
    fn from(error: EncodeError) -> Self {
        Self::EncodeError(error)
    }
}

impl From<DecodeError> for AppError {
    fn from(error: DecodeError) -> Self {
        Self::DecodeError(error)
    }
}

impl<L: Error + 'static> From<TranscodeError<L>> for AppError {
    fn from(error: TranscodeError<L>) -> Self {
        Self::TranscodeError(error.into_dynamic())
    }
}

impl From<DownloadToFileError> for AppError {
    fn from(error: DownloadToFileError) -> Self {
        match error {
            DownloadToFileError::IoError(error) => Self::IoError(error),
            DownloadToFileError::HttpError(error) => Self::DictDownloadError(Box::new(error)),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(error) => write!(f, "[I/O error] {error}"),
            AppError::CliError(error) => Display::fmt(error, f),
            AppError::EncodeError(error) => write!(f, "[encode error] {error}"),
            AppError::DecodeError(error) => write!(f, "[decode error] {error}"),
            AppError::TranscodeError(error) => write!(f, "[transcode error] {error}"),
            AppError::DictDownloadError(error) => write!(f, "[dict. download error] {error}"),
        }
    }
}

impl Error for AppError {}