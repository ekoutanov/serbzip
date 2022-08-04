use std::{error, fs, io, process};
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, Read, Write};
use std::path::Path;
use serbzip::codec::{compress_line, expand_line};
use serbzip::codec::dict::Dict;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut reader = BufReader::new(File::open("dict.txt")?);
    let dict = Dict::read_from_text_file(&mut reader)?;
    let mut writer = BufWriter::new(File::create("dict.img")?);
    dict.write_to_binary_image(&mut writer)?;
    writer.flush().unwrap();
    drop(writer);

    // let mut reader = BufReader::new(File::open("dict.img")?);
    // let dict = Dict::load_from_binary_image(&mut reader)?;
    // println!("Loaded dict with {} words", dict.count());
    // compress_from_stdin(&dict);
    expand_from_stdin(&dict);
    Ok(())
}

fn compress_from_stdin(dict: &Dict) {
    // let mut read_buf = String::new();
    // loop {
    //     match io::stdin().read_line(&mut read_buf) {
    //         Ok(0) => {
    //             process::exit(0)
    //         }
    //         Ok(size) => {
    //             let compressed = compress_line(&dict, &read_buf[0..size-1]);
    //             println!("{}", compressed);
    //             read_buf.clear();
    //         }
    //         Err(_) => {
    //             process::exit(1)
    //         }
    //     }
    // }
    process_from_stdin(|line| compress_line(&dict, line));
}

fn expand_from_stdin(dict: &Dict) {
    process_from_stdin(|line| expand_line(&dict, line));
}

fn process_from_stdin(mut processor: impl FnMut(&str) -> String) {
    let mut read_buf = String::new();
    loop {
        match io::stdin().read_line(&mut read_buf) {
            Ok(0) => {
                process::exit(0)
            }
            Ok(size) => {
                let output = processor(&read_buf[0..size-1]);
                println!("{}", output);
                read_buf.clear();
            }
            Err(_) => {
                process::exit(1)
            }
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