//! Entry point for the Balkanoid codec.

use crate::cli::app_error::{AppError, CliError, CliErrorDetail, CliErrorKind};
use crate::cli::banner::{BLUE, RED, WHITE};
use crate::cli::{banner, downloader, is_extension, Args, Mode, compress_helper, expand_helper};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use serbzip::succinct::CowStr;

const DICT_EXT_BINARY: &str = "blk";
const DICT_EXT_TEXT: &str = "txt";
const HOME_DICT_FILE: &str = ".serbzip/dict.blk";
const DEFAULT_DICT_BINARY_FILE: &str = "dict.blk";
const DEFAULT_DICT_TEXT_FILE: &str = "dict.txt";
const DICT_URL: &str = "https://github.com/ekoutanov/serbzip/raw/master/dict.blk";

pub(super) fn run(args: &Args) -> Result<(), AppError> {
    if !args.quiet {
        banner::print(
            r#"

██████╗  █████╗ ██╗     ██╗  ██╗ █████╗ ███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗██║     ██║ ██╔╝██╔══██╗████╗  ██║██╔═══██╗██║██╔══██╗
██████╔╝███████║██║     █████╔╝ ███████║██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██╗██╔══██║██║     ██╔═██╗ ██╔══██║██║╚██╗██║██║   ██║██║██║  ██║
██████╔╝██║  ██║███████╗██║  ██╗██║  ██║██║ ╚████║╚██████╔╝██║██████╔╝
╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

    "#,
            &[RED, RED, BLUE, BLUE, WHITE, WHITE],
        );
    }

    // read the dictionary from either the user-supplied or default path
    let dict = {
        let dict_path = args.dict_path()?;
        let unsupported_format_err = || {
            AppError::from(CliError(CliErrorKind::UnsupportedDictFormat, CliErrorDetail::Owned(format!("unsupported dictionary format for {dict_path:?}: only .{DICT_EXT_TEXT} and .{DICT_EXT_BINARY} files can be read"))))
        };
        match args.dict_path()?.extension() {
            None => Err(unsupported_format_err()),
            Some(extension) => match extension.to_ascii_lowercase().to_string_lossy().borrow() {
                DICT_EXT_TEXT => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_text_file(&mut reader)?)
                }
                DICT_EXT_BINARY => {
                    let mut reader = BufReader::new(File::open(dict_path)?);
                    Ok(Dict::read_from_binary_image(&mut reader)?)
                }
                _ => Err(unsupported_format_err()),
            },
        }?
    };

    match args.mode()? {
        Mode::Compress => compress_helper(args, &Balkanoid::new(&dict)),
        Mode::Expand => expand_helper(args, &Balkanoid::new(&dict)),
        Mode::Compile => { // if the imaging option has been set, serialize dict to a user-specified file
            match args.dictionary_image_output_file() {
                None => Err(AppError::from(CliError(CliErrorKind::UnspecifiedBinaryDictOutputFile, CowStr::Borrowed("dictionary output file not specified")))),
                Some(image_output_file) => {
                    if !is_extension(image_output_file, DICT_EXT_BINARY) {
                        return Err(AppError::from(CliError(
                            CliErrorKind::UnsupportedBinaryDictFormat,
                            CliErrorDetail::Owned(
                                format!("only .{DICT_EXT_BINARY} files are supported for compiled dictionaries"),
                            ),
                        )));
                    }
                    if !args.quiet() {
                        eprintln!(
                            "Writing compiled dictionary image to {image_output_file} ({words} words)",
                            words = dict.count()
                        );
                    }
                    let mut writer = BufWriter::new(File::create(image_output_file)?);
                    dict.write_to_binary_image(&mut writer)?;
                    writer.flush()?;
                    Ok(())
                }
            }
        }
    }
}

impl Args {
    fn dict_path(&self) -> Result<PathBuf, AppError> {
        match &self.dictionary {
            None => {
                let local_img_path = Path::new(DEFAULT_DICT_BINARY_FILE);
                if local_img_path.exists() {
                    Ok(local_img_path.to_owned())
                } else {
                    let local_txt_path = Path::new(DEFAULT_DICT_TEXT_FILE);
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
                                    if !self.quiet() { eprintln!("Downloading dictionary file to {home_img_path:?}"); }
                                    downloader::download_to_file(DICT_URL, home_img_path.clone())?;
                                    if !self.quiet() { eprintln!("Download complete"); }
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
}
