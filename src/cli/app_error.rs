//! Errors that may arise in the CLI.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use bincode::error::{DecodeError, EncodeError};
use serbzip::codecs::balkanoid::dict::{ReadFromTextFileError, OverflowError};
use serbzip::succinct::CowStr;
use serbzip::transcoder::TranscodeError;
use crate::cli::downloader::DownloadToFileError;

/// Errors arising from the parsing of CLI args.
#[derive(Debug)]
pub enum CliErrorKind {
    UnsupportedDictFormat,
    UnsupportedBinaryDictFormat,
    UnspecifiedBinaryDictOutputFile,
    InvalidMode,
    UnsupportedMode,
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

/// A complete listing of all application-level errors. This includes errors trapped during
/// the parsing of command-line arguments, I/O errors and errors emitted by the codec.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    DictDownloadHttp(Box<dyn Error>),
    DictDownloadStatusNotOk(u16),
    Encode(EncodeError),
    Decode(DecodeError),
    Cli(CliError),
    Transcode(TranscodeError<Box<dyn Error>>),
    DictOverflow(OverflowError)
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<CliError> for AppError {
    fn from(error: CliError) -> Self {
        Self::Cli(error)
    }
}

impl From<EncodeError> for AppError {
    fn from(error: EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<DecodeError> for AppError {
    fn from(error: DecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<ReadFromTextFileError> for AppError {
    fn from(error: ReadFromTextFileError) -> Self {
        match error {
            ReadFromTextFileError::Io(error) => Self::Io(error),
            ReadFromTextFileError::DictOverflow(error) => Self::DictOverflow(error)
        }
    }
}

impl<L: Error + 'static> From<TranscodeError<L>> for AppError {
    fn from(error: TranscodeError<L>) -> Self {
        Self::Transcode(error.into_dynamic())
    }
}

impl From<DownloadToFileError> for AppError {
    fn from(error: DownloadToFileError) -> Self {
        match error {
            DownloadToFileError::Io(error) => Self::Io(error),
            DownloadToFileError::Http(error) => Self::DictDownloadHttp(Box::new(error)),
            DownloadToFileError::StatusNotOk(status) => Self::DictDownloadStatusNotOk(status),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(error) => write!(f, "[I/O error] {error}"),
            AppError::Cli(error) => Display::fmt(error, f),
            AppError::Encode(error) => write!(f, "[encode error] {error}"),
            AppError::Decode(error) => write!(f, "[decode error] {error}"),
            AppError::Transcode(error) => write!(f, "[transcode error] {error}"),
            AppError::DictDownloadHttp(error) => write!(f, "[dict. download error] {error}"),
            AppError::DictDownloadStatusNotOk(status) => write!(f, "[dict. download status not 200/OK] {status}"),
            AppError::DictOverflow(error) => write!(f, "[dict. overflow error] {error}"),
        }
    }
}

impl Error for AppError {}