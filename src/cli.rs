use bincode::error::{DecodeError, EncodeError};
use clap::Parser;
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use serbzip::succinct::{CowStr, Errorlike};
use serbzip::transcoder::TranscodeError;
use std::borrow::Borrow;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, io};
use banner::{BLUE, RED, WHITE, YELLOW};
use serbzip::codecs::armenoid::Armenoid;

pub mod banner;

const HOME_DICT_FILE: &str = ".serbzip/dict.img";
const DICT_URL: &str = "https://github.com/ekoutanov/serbzip/raw/master/dict.img";

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
pub struct CliError(CliErrorKind, CliErrorDetail);

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

impl From<DownloadAndSaveError> for AppError {
    fn from(error: DownloadAndSaveError) -> Self {
        match error {
            DownloadAndSaveError::IoError(error) => Self::IoError(error),
            DownloadAndSaveError::HttpError(error) => Self::DictDownloadError(Box::new(error)),
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

enum DownloadAndSaveError {
    IoError(io::Error),
    HttpError(reqwest::Error)
}

impl From<io::Error> for DownloadAndSaveError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<reqwest::Error> for DownloadAndSaveError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

fn create_parent_dirs(path: &impl AsRef<Path>) -> Result<(), io::Error> {
    let path = path.as_ref();
    create_dir_all(path.parent().unwrap())?;
    Ok(())
}

fn download_and_safe_file(url: &str, path: impl AsRef<Path>) -> Result<(), DownloadAndSaveError> {
    let resp = reqwest::blocking::get(url)?;
    let body = resp.bytes()?;
    let mut body_bytes = &body[..];
    create_parent_dirs(&path)?;
    let mut out = File::create(path)?;
    io::copy(&mut body_bytes, &mut out)?;
    Ok(())
}

pub fn run() -> Result<(), AppError> {
    let args = Args::from(&mut env::args_os());
    match args.codec.clone().unwrap_or(CodecImpl::Balkanoid) {
        CodecImpl::Balkanoid => run_balkanoid(args),
        CodecImpl::Armenoid => run_armenoid(args),
    }
}

fn run_balkanoid(args: Args) -> Result<(), AppError> {
    banner::print(r#"

██████╗  █████╗ ██╗     ██╗  ██╗ █████╗ ███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗██║     ██║ ██╔╝██╔══██╗████╗  ██║██╔═══██╗██║██╔══██╗
██████╔╝███████║██║     █████╔╝ ███████║██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██╗██╔══██║██║     ██╔═██╗ ██╔══██║██║╚██╗██║██║   ██║██║██║  ██║
██████╔╝██║  ██║███████╗██║  ██╗██║  ██║██║ ╚████║╚██████╔╝██║██████╔╝
╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

    "#, &[RED, RED, BLUE, BLUE, WHITE, WHITE]);

    // read the dictionary from either the user-supplied or default path
    let dict = {
        let dict_path = args.dict_path()?;
        match args.dict_path()?.extension() {
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
        }?
    };

    // if the imaging option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = args.dictionary_image_output_file() {
        if !is_extension(image_output_file, "img") {
            return Err(AppError::from(CliError(
                CliErrorKind::UnsupportedBinaryDictionaryFormat,
                CliErrorDetail::Borrowed("only .img files are supported for compiled dictionaries"),
            )));
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

fn run_armenoid(args: Args) -> Result<(), AppError> {
    banner::print(r#"

 █████╗ ██████╗ ███╗   ███╗███████╗███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗████╗ ████║██╔════╝████╗  ██║██╔═══██╗██║██╔══██╗
███████║██████╔╝██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██║██╔══██╗██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║██║██║  ██║
██║  ██║██║  ██║██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝██║██████╔╝
╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

    "#, &[RED, RED, BLUE, BLUE, YELLOW, YELLOW]);

    let mode = args.mode()?;
    let input_reader = args.input_reader()?;
    let mut output_writer: Box<dyn Write> = args.output_writer()?;
    let codec = Armenoid::default();
    match mode {
        Mode::Compress => codec.compress(&mut BufReader::new(input_reader), &mut output_writer)?,
        Mode::Expand => codec.expand(&mut BufReader::new(input_reader), &mut output_writer)?,
    }
    output_writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
enum CodecImpl {
    Balkanoid,
    Armenoid,
}

impl FromStr for CodecImpl {
    type Err = Errorlike<CowStr>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "balkanoid" => Ok(Self::Balkanoid),
            "armenoid" => Ok(Self::Armenoid),
            other => Err(Errorlike::from_owned(format!("no such codec '{other}'"))),
        }
    }
}

/// A quasi-lossless Balkanoidal meta-lingual compressor.
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

    /// Codec implementation (defaults to balkanoid)
    #[clap(long, value_parser)]
    codec: Option<CodecImpl>,
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
            (true, true) | (false, false) => Err(CliError(
                CliErrorKind::InvalidMode,
                CliErrorDetail::Borrowed("either one of --compress or --expand must be specified"),
            )),
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
                    Err(AppError::from(CliError(
                        CliErrorKind::NoSuchInputFile,
                        CliErrorDetail::Owned(format!("failed to open input file {path:?}")),
                    )))
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
                                let home_img_path = path.join(Path::new(HOME_DICT_FILE));
                                if home_img_path.exists() {
                                    Ok(home_img_path)
                                } else {
                                    eprintln!("Downloading dictionary file to {home_img_path:?}");
                                    download_and_safe_file(DICT_URL, home_img_path.clone())?;
                                    Ok(home_img_path)
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
                    Err(AppError::from(CliError(
                        CliErrorKind::NoSuchDictFile,
                        CliErrorDetail::Owned(format!(
                            "failed to open dictionary file {specified_path:?}"
                        )),
                    )))
                }
            }
        }
    }

    fn dictionary_image_output_file(&self) -> &Option<String> {
        &self.dictionary_image_output_file
    }
}

fn is_extension(filename: impl AsRef<Path>, ext: &str) -> bool {
    let filename = filename.as_ref();
    filename
        .extension()
        .map_or(false, |file_ext| file_ext.eq_ignore_ascii_case(ext))
}
