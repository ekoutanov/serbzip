use std::{fs, io, process};
use std::io::{Error, Read};
use std::path::Path;
use serbzip::codec::{compress_line, Dict};

fn main() {
    let mut read_buf = String::new();
    let dict = build_dict("dict.txt").unwrap();
    println!("Loaded dict with {} words", dict.count());

    loop {
        match io::stdin().read_line(&mut read_buf) {
            Ok(0) => {
                process::exit(0)
            }
            Ok(_) => {
                let compressed = compress_line(&dict, &read_buf);
                print!("{}", compressed);
                read_buf.clear();
            }
            Err(_) => {
                process::exit(1)
            }
        }
    }
}

fn build_dict<P: AsRef<Path>>(path: P) -> Result<Dict, ReadError> {
    let contents = read_file(path)?;
    let mut dict = Dict::default();
    for line in contents.lines() {
        let line = line.split_whitespace();
        let line = line.map(ToOwned::to_owned).collect::<Vec<_>>();
        dict.populate(&line);
    }
    Ok(dict)
}

#[derive(Debug)]
enum ReadError {
    NoSuchFile,
    IoError(io::Error)
}

impl From<io::Error> for ReadError {
    fn from(error: Error) -> Self {
        ReadError::IoError(error)
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, ReadError> {
    let path = path.as_ref();
    let exists = path.try_exists()?;
    if !exists {
        return Err(ReadError::NoSuchFile)
    }

    Ok(fs::read_to_string(path)?)
}