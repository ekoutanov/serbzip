use std::error::Error;
use clap::Parser;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::{env, io};
use std::borrow::Borrow;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use bincode::error::{DecodeError, EncodeError};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use serbzip::succinct::CowStr;
use serbzip::transcoder::TranscodeError;

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

pub type CliErrorDetail = CowStr;

#[derive(Debug)]
pub struct CliError(CliErrorKind, CliErrorDetail);

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.0, self.1)
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

pub fn run() -> Result<(), AppError> {
    let args = Args::from(&mut env::args_os());
    //eprintln!("args: {args:?}");

    let dict_path = args.dict_path()?;

    // read the dictionary from either the user-supplied or default path
    let dict = match dict_path.extension() {
        None => {
            Err(AppError::from(CliError(CliErrorKind::UnsupportedDictionaryFormat, CliErrorDetail::Owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read")))))
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
                _ =>  Err(AppError::from(CliError(CliErrorKind::UnsupportedDictionaryFormat, CliErrorDetail::Owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read")))))
            }
        }
    }?;

    // if the imaging option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = args.dictionary_image_output_file() {
        if !is_extension(image_output_file, "img") {
            return Err(AppError::from(CliError(CliErrorKind::UnsupportedBinaryDictionaryFormat, CliErrorDetail::Borrowed(
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

/// A quasi-lossless Balkanoidal meta-lingual compressor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Compress
    #[clap(short, long, value_parser)]
    compress: bool,

    /// Expand
    #[clap(short = 'x', long, value_parser)]
    expand: bool,

    /// Dictionary file (.txt for a text file, .img for binary image)
    #[clap(short, long, value_parser)]
    dictionary: Option<String>,

    /// Input file (defaults to stdin)
    #[clap(short, long, value_parser)]
    input_file: Option<String>,

    /// Output file (defaults to stdout)
    #[clap(short, long, value_parser)]
    output_file: Option<String>,

    /// Output file for compiling the dictionary image
    #[clap(short = 'm', long, value_parser)]
    dictionary_image_output_file: Option<String>,
}

#[derive(Debug)]
enum Mode {
    Compress,
    Expand,
}

impl Args {
    pub fn from<I, T>(itr: I) -> Args
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Args::parse_from(itr)
    }

    fn mode(&self) -> Result<Mode, CliError> {
        match (self.compress, self.expand) {
            (true, true) | (false, false) => Err(CliError(CliErrorKind::InvalidMode, CliErrorDetail::Borrowed(
                "either one of --compress or --expand must be specified",
            ))),
            (true, false) => Ok(Mode::Compress),
            (false, true) => Ok(Mode::Expand),
        }
    }

    fn input_reader(&self) -> Result<Box<dyn Read>, AppError> {
        self.input_file
            .as_ref()
            .map_or(Ok(Box::new(io::stdin())), |path| {
                let path = Path::new(&path);
                if path.exists() {
                    Ok(Box::new(File::open(path)?))
                } else {
                    Err(AppError::from(CliError(CliErrorKind::NoSuchInputFile, CliErrorDetail::Owned(format!(
                        "failed to open input file {path:?}"
                    )))))
                }
            })
    }

    fn output_writer(&self) -> Result<Box<dyn Write>, io::Error> {
        self.output_file
            .as_ref()
            .map_or(Ok(Box::new(io::stdout())), |path| {
                Ok(Box::new(BufWriter::new(File::create(path)?)))
            })
    }

    fn dict_path(&self) -> Result<PathBuf, AppError> {
        match &self.dictionary {
            None => {
                let local_img_path = Path::new("dict.img");
                if local_img_path.exists() {
                    Ok(local_img_path.to_owned())
                } else {
                    let local_txt_path = Path::new("dict.txt");
                    if local_txt_path.exists() {
                        Ok(local_txt_path.to_owned())
                    } else {
                        // resolved to ${HOME} (in *nix-based systems) or %USERPROFILE% in Windows
                        match home::home_dir() {
                            None => {
                                Err(AppError::from(CliError(CliErrorKind::NoHomeDir, CliErrorDetail::Borrowed("the user's home directory could not be located; please specify the dictionary file"))))
                            }
                            Some(path) => {
                                let home_img_path = path.as_path().join(Path::new("/.serbzip/dict.img"));
                                if home_img_path.exists() {
                                    Ok(home_img_path)
                                } else {
                                    Err(AppError::from(CliError(CliErrorKind::NoDefaultDict, CliErrorDetail::Borrowed("no dict.img in ~/.serbzip; please specify the dictionary file"))))
                                }
                            }
                        }
                    }
                }
            }
            Some(path) => {
                let specified_path = Path::new(&path);
                if specified_path.exists() {
                    Ok(specified_path.to_owned())
                } else {
                    Err(AppError::from(CliError(CliErrorKind::NoSuchDictFile, CliErrorDetail::Owned(format!(
                        "failed to open dictionary file {specified_path:?}"
                    )))))
                }
            }
        }
    }

    fn dictionary_image_output_file(&self) -> &Option<String> {
        &self.dictionary_image_output_file
    }
}

fn is_extension<P>(filename: P, ext: &str) -> bool where P: Into<PathBuf> {
    let filename = filename.into();
    filename.extension()
        .map_or(false, |file_ext| file_ext.eq_ignore_ascii_case(ext))
}
