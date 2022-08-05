use std::{error, io, process};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};
use std::path::Path;

use clap::Parser;
use home;
use serbzip::codec;

use serbzip::codec::{compress_line, expand_line};
use serbzip::codec::dict::Dict;
use serbzip::transcoder;
use serbzip::transcoder::TranscodeError;

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

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut args = Args::parse() as Args;
    //eprintln!("args: {args:?}");

    let dict_path = args.dictionary.take()
        .map(|path| {
            let specified_path = Path::new(&path);
            if specified_path.exists() {
                specified_path.to_owned()
            } else {
                eprintln!("Failed to open dictionary file {specified_path:?}");
                process::exit(1);
            }
        })
        .or_else(|| {
            let local_img_path = Path::new("dict.img");
            if local_img_path.exists() {
                Some(local_img_path.to_owned())
            } else {
                let local_txt_path = Path::new("dict.txt");
                if local_txt_path.exists() {
                    Some(local_txt_path.to_owned())
                } else {
                    // resolved to ${HOME} (in *nix-based systems) or %USERPROFILE% in Windows
                    match home::home_dir() {
                        None => {
                            eprintln!("The user's home directory could not be located; please specify the dictionary file");
                            process::exit(1);
                        }
                        Some(path) => {
                            let home_img_path = path.as_path().join(Path::new("/dict.img"));
                            if home_img_path.exists() {
                                Some(home_img_path.to_owned())
                            } else {
                                eprintln!("The user's home directory could not be located; please specify the dictionary file");
                                process::exit(1);
                            }
                        }
                    }
                }
            }
        }).unwrap();

    // read the dictionary from either the user-supplied or default path
    let dict = match dict_path.extension() {
        None => {
            eprintln!("Unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read");
            process::exit(1);
        }
        Some(extension) => {
            if extension == "txt" {
                let mut reader = BufReader::new(File::open(dict_path)?);
                Dict::read_from_text_file(&mut reader)?
            } else if extension == "img" {
                let mut reader = BufReader::new(File::open(dict_path)?);
                Dict::read_from_binary_image(&mut reader)?
            } else {
                eprintln!("Unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read");
                process::exit(1);
            }
        }
    };

    // if the -m option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = &args.dictionary_image_output_file {
        if !image_output_file.to_string().ends_with(".img") {
            eprintln!("Only .img files are supported for compiled dictionaries");
            process::exit(1);
        }
        eprintln!("Writing compiled dictionary image to {image_output_file}");
        let mut writer = BufWriter::new(File::create(image_output_file)?);
        dict.write_to_binary_image(&mut writer)?;
        writer.flush().unwrap();
        drop(writer);
        process::exit(0);
    }

    if !(args.compress ^ args.expand) {
        eprintln!("Either one of --compress or --expand must be specified");
        process::exit(1)
    }

    let input_reader: Box<dyn Read> = args.input_file.map_or(Box::new(io::stdin()), |path| {
        let path = Path::new(&path);
        if !path.exists() {
            eprintln!("Failed to open input file {path:?}");
            process::exit(1)
        }
        Box::new(File::open(path).unwrap())
    });

    let output_writer: Box<dyn Write> = args.output_file.map_or(Box::new(io::stdout()), |path| {
        Box::new(File::create(path).unwrap())
    });

    if args.compress {
        compress(&dict, &mut BufReader::new(input_reader), &mut BufWriter::new(output_writer))?;
    } else {
        expand(&dict, &mut BufReader::new(input_reader), &mut BufWriter::new(output_writer))?;
    }
    Ok(())
}

fn compress(dict: &Dict, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), TranscodeError> {
    transcoder::transcode(r, w, |_, line| Ok(codec::compress_line(&dict, line)))
}

fn expand(dict: &Dict, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), TranscodeError> {
    transcoder::transcode(r, w, |_, line| codec::expand_line(&dict, line))
}