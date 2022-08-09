use clap::Parser;
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use serbzip::succinct::{CowStr, Errorlike};
use std::borrow::Borrow;
use std::ffi::OsString;
use std::fmt::{Debug};
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, io};
use banner::{BLUE, RED, WHITE, YELLOW};
use serbzip::codecs::armenoid::Armenoid;
use crate::cli::app_error::{AppError, CliError, CliErrorDetail, CliErrorKind};

pub mod banner;
mod downloader;
pub mod app_error;

const HOME_DICT_FILE: &str = ".serbzip/dict.img";
const DICT_URL: &str = "https://github.com/ekoutanov/serbzip/raw/master/dict.img";

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
                                    downloader::download_to_file(DICT_URL, home_img_path.clone())?;
                                    eprintln!("Download complete");
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

fn is_extension(filename: &impl AsRef<Path>, ext: &str) -> bool {
    filename.as_ref()
        .extension()
        .map_or(false, |file_ext| file_ext.eq_ignore_ascii_case(ext))
}
