use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::{env, io, process};
use std::borrow::{Borrow};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use bincode::error::{DecodeError, EncodeError};

use crate::cli::{Args, Mode};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use serbzip::transcoder::TranscodeError;
use crate::succinct::{CowStr, Errorlike};

mod cli;
mod succinct;

pub type ArgsError = Errorlike<CowStr>;

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    ArgsError(ArgsError),
    TranscodeError(TranscodeError<Box<dyn Error>>)
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<ArgsError> for AppError {
    fn from(error: ArgsError) -> Self {
        Self::ArgsError(error)
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

// impl From<TranscodeError<Box<dyn Error>>> for AppError {
//     fn from(error: TranscodeError<Box<dyn Error>>) -> Self {
//         Self::TranscodeError(error)
//     }
// }

impl <L: Error + 'static> From<TranscodeError<L>> for AppError {
    fn from(error: TranscodeError<L>) -> Self {
        Self::TranscodeError(error.boxify())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(error) => Debug::fmt(error, f),
            AppError::ArgsError(error) => Display::fmt(error, f),
            AppError::EncodeError(error) => Display::fmt(error, f),
            AppError::DecodeError(error) => Display::fmt(error, f),
            AppError::TranscodeError(error) => Display::fmt(error, f)
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
            Err(AppError::from(ArgsError::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read"))))
        }
        Some(extension) => {
            match extension.to_string_lossy().borrow() {
                "txt" => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_text_file(&mut reader)?)
                }
                "img" => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_binary_image(&mut reader)?)
                }
                _ =>  Err(AppError::from(ArgsError::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read"))))
            }
        }
    }?;

    // if the imaging option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = args.dictionary_image_output_file() {
        if !image_output_file.to_string().ends_with(".img") {
            return Err(AppError::from(ArgsError::from_borrowed("only .img files are supported for compiled dictionaries")))
        }
        eprintln!(
            "Writing compiled dictionary image to {image_output_file} ({words} words)",
            words = dict.count()
        );
        let mut writer = BufWriter::new(File::create(image_output_file)?);
        dict.write_to_binary_image(&mut writer)?;
        writer.flush()?;
        return Ok(())
    }

    let mode = args.mode()?;
    let input_reader = args.input_reader()?;
    let mut output_writer: Box<dyn Write> = args.output_writer()?;

    let codec = Balkanoid::new(&dict);
    match mode {
        Mode::Compress => codec.compress(&mut BufReader::new(input_reader), &mut output_writer)?,
        Mode::Expand => codec.expand(&mut BufReader::new(input_reader), &mut output_writer)?,
        // Mode::Expand => codec.expand(&mut BufReader::new(input_reader), &mut output_writer).map_err(TranscodeError::boxify)?,
    }
    output_writer.flush()?;
    Ok(())
}
