use clap::Parser;
use home;
use serbzip::codec::dict::Dict;
use serbzip::codec::{compress_line, expand_line};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, Read, Write};
use std::path::{Path, PathBuf};
use std::{error, fs, io, process};

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

    /// Input file (or '-' for stdin)
    #[clap(short, long, value_parser)]
    input_file: Option<String>,

    /// Output file (or '-' for stdout)
    #[clap(short, long, value_parser)]
    output_file: Option<String>,

    /// Output file for compiling the dictionary image
    #[clap(short = 'm', long, value_parser)]
    dictionary_image_output_file: Option<String>,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut args = Args::parse();
    //eprintln!("args: {args:?}");

    let dict_file: Option<String> = args.dictionary.take();
    let dict_path = dict_file
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

    if args.compress {
        compress_from_stdin(&dict);
    } else {
        expand_from_stdin(&dict);
    }
    Ok(())
}

fn compress_from_stdin(dict: &Dict) {
    process_from_stdin(|line| compress_line(&dict, line));
}

fn expand_from_stdin(dict: &Dict) {
    process_from_stdin(|line| expand_line(&dict, line));
}

fn process_from_stdin(mut processor: impl FnMut(&str) -> String) {
    let mut read_buf = String::new();
    loop {
        match io::stdin().read_line(&mut read_buf) {
            Ok(0) => process::exit(0),
            Ok(size) => {
                let output = processor(&read_buf[0..size - 1]);
                println!("{}", output);
                read_buf.clear();
            }
            Err(_) => process::exit(1),
        }
    }
}

// fn build_dict<P: AsRef<Path>>(path: P) -> Result<Option<Dict>, io::Error> {
//     match read_file(path)? {
//         None => Ok(None),
//         Some(contents) => {
//             let mut dict = Dict::default();
//             for line in contents.lines() {
//                 let line = line.split_whitespace();
//                 // let line = line.map(ToOwned::to_owned).collect::<Vec<_>>();
//                 dict.populate(line.map(ToOwned::to_owned));
//             }
//             Ok(Some(dict))
//         }
//     }
// }

// #[derive(Debug)]
// enum ReadError {
//     NoSuchFile,
//     IoError(io::Error)
// }
//
// impl From<io::Error> for ReadError {
//     fn from(error: Error) -> Self {
//         ReadError::IoError(error)
//     }
// }

// fn read_file<P: AsRef<Path>>(path: P) -> Result<Option<String>, io::Error> {
//     let path = path.as_ref();
//     let exists = path.try_exists()?;
//     if !exists {
//         return Ok(None)
//     }
//
//     Ok(Some(fs::read_to_string(path)?))
// }
