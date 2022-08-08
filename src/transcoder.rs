use std::{error, io};
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, Write};
use crate::succinct::{CowStr, Errorlike};

pub type LineProcessingError = Errorlike<CowStr>;

#[derive(Debug)]
pub enum TranscodeError {
    ProcessingError { line_no: u32, error: LineProcessingError },
    IoError(io::Error)
}

impl Display for TranscodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl error::Error for TranscodeError {}

impl From<io::Error> for TranscodeError {
    fn from(error: io::Error) -> Self {
        TranscodeError::IoError(error)
    }
}

pub fn transcode(
    r: &mut impl BufRead,
    w: &mut impl Write,
    mut processor: impl FnMut(u32, &str) -> Result<String, LineProcessingError>,
) -> Result<(), TranscodeError> {
    let mut read_buf = String::new();
    let mut line_no = 1u32;
    loop {
        match r.read_line(&mut read_buf)? {
            0 => return Ok(()),
            size => {
                match processor(line_no, &read_buf[0..size - 1]) {
                    Ok(output) => {
                        writeln!(w, "{}", output)?;
                        read_buf.clear();
                    }
                    Err(error) => {
                        return Err(TranscodeError::ProcessingError { line_no, error })
                    }
                }
            }
        }
        line_no += 1;
    }
}