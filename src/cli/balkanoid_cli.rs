use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use crate::cli::app_error::{AppError, CliError, CliErrorDetail, CliErrorKind};
use crate::cli::{Args, banner, downloader, is_extension, Mode};
use crate::cli::banner::{BLUE, RED, WHITE};

const HOME_DICT_FILE: &str = ".serbzip/dict.img";
const DICT_URL: &str = "https://github.com/ekoutanov/serbzip/raw/master/dict.img";

pub (in super) fn run(args: Args) -> Result<(), AppError> {
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

impl Args {
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
}