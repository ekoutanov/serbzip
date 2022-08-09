use clap::Parser;
use serbzip::succinct::{CowStr, Errorlike};
use std::ffi::OsString;
use std::fmt::{Debug};
use std::fs::{File};
use std::io::{BufWriter, Read, Write};
use std::str::FromStr;
use std::{env, io};
use std::path::Path;
use crate::cli::app_error::{AppError, CliError, CliErrorDetail, CliErrorKind};

pub mod banner;
pub mod app_error;
mod downloader;
mod balkanoid_cli;
mod armenoid_cli;

pub fn run() -> Result<(), AppError> {
    let args = Args::from(&mut env::args_os());
    match args.codec.clone().unwrap_or(CodecImpl::Balkanoid) {
        CodecImpl::Balkanoid => balkanoid_cli::run(args),
        CodecImpl::Armenoid => armenoid_cli::run(args),
    }
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

    fn dictionary_image_output_file(&self) -> &Option<String> {
        &self.dictionary_image_output_file
    }
}

fn is_extension(filename: &impl AsRef<Path>, ext: &str) -> bool {
    filename.as_ref()
        .extension()
        .map_or(false, |file_ext| file_ext.eq_ignore_ascii_case(ext))
}
