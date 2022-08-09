use bincode::error::{DecodeError, EncodeError};
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::{env, io, process};
use std::path::{PathBuf};

use crate::cli::{Args, Mode};
use crate::succinct::{CowStr, Errorlike};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use serbzip::transcoder::TranscodeError;

mod cli;
mod succinct;

#[derive(Debug)]
pub enum CliErrorKind {
    UnsupportedDictionaryFormat,
    UnsupportedBinaryDictionaryFormat,
    InvalidMode,
    NoSuchInputFile,
    NoHomeDir,
    NoDefaultDict,
    NoSuchDictFile,
}

pub type CliErrorDetail = Errorlike<CowStr>;

#[derive(Debug)]
pub struct CliError(CliErrorKind, CliErrorDetail);

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind: {:?}, detail: {}", self.0, self.1)
    }
}

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
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

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(error) => Debug::fmt(error, f),
            AppError::CliError(error) => Display::fmt(error, f),
            AppError::EncodeError(error) => Display::fmt(error, f),
            AppError::DecodeError(error) => Display::fmt(error, f),
            AppError::TranscodeError(error) => Display::fmt(error, f),
        }
    }
}

impl Error for AppError {}

fn main() {
    run().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        process::exit(1)
    });
}

fn run() -> Result<(), AppError> {
    let args = Args::from(&mut env::args_os());
    //eprintln!("args: {args:?}");

    let dict_path = args.dict_path()?;

    // read the dictionary from either the user-supplied or default path
    let dict = match dict_path.extension() {
        None => {
            Err(AppError::from(CliError(CliErrorKind::UnsupportedDictionaryFormat, CliErrorDetail::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read")))))
        }
        Some(extension) => {
            match extension.to_ascii_lowercase().to_string_lossy().borrow() {
                "txt" => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_text_file(&mut reader)?)
                }
                "img" => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_binary_image(&mut reader)?)
                }
                _ =>  Err(AppError::from(CliError(CliErrorKind::UnsupportedDictionaryFormat, CliErrorDetail::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read")))))
            }
        }
    }?;

    // if the imaging option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = args.dictionary_image_output_file() {
        if !is_extension(image_output_file, "img") {
            return Err(AppError::from(CliError(CliErrorKind::UnsupportedBinaryDictionaryFormat, CliErrorDetail::from_borrowed(
                "only .img files are supported for compiled dictionaries",
            ))));
        }
        eprintln!(
            "Writing compiled dictionary image to {image_output_file} ({words} words)",
            words = dict.count()
        );
        let mut writer = BufWriter::new(File::create(image_output_file)?);
        dict.write_to_binary_image(&mut writer)?;
        writer.flush()?;
        return Ok(());
    }

    let mode = args.mode()?;
    let input_reader = args.input_reader()?;
    let mut output_writer: Box<dyn Write> = args.output_writer()?;

    let codec = Balkanoid::new(&dict);
    match mode {
        Mode::Compress => codec.compress(&mut BufReader::new(input_reader), &mut output_writer)?,
        Mode::Expand => codec.expand(&mut BufReader::new(input_reader), &mut output_writer)?,
    }
    output_writer.flush()?;
    Ok(())
}

fn is_extension<P>(filename: P, ext: &str) -> bool where P: Into<PathBuf> {
    let filename = filename.into();
    filename.extension()
        .map_or(false, |file_ext| file_ext.eq_ignore_ascii_case(ext))
}
