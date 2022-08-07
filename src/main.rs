use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::{env, error, process};
use std::borrow::{Borrow, Cow};

use crate::cli::{Args, Mode};
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use crate::succinct::{CowStr, Errorlike};

mod cli;
mod succinct;

fn main() {
    run().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        process::exit(1)
    });
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let args = Args::from(&mut env::args_os());
    //eprintln!("args: {args:?}");

    let dict_path = args.dict_path()?;

    // read the dictionary from either the user-supplied or default path
    let dict = match dict_path.extension() {
        None => {
            Err(Box::new(Errorlike::<CowStr>::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read"))))
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
                _ =>  Err(Box::new(Errorlike::from_owned(format!("unsupported dictionary format for {dict_path:?}: only .txt and .img files can be read"))))
            }
        }
    }?;

    // if the imaging option has been set, serialize dict to a user-specified file
    if let Some(image_output_file) = args.dictionary_image_output_file() {
        if !image_output_file.to_string().ends_with(".img") {
            return Err(Box::new(Errorlike::from_borrowed("only .img files are supported for compiled dictionaries")))
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
    }
    output_writer.flush()?;
    Ok(())
}
