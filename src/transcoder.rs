use crate::succinct::{CowStr, Errorlike};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, Write};
use std::{error, io};

pub type LineProcessingError = Errorlike<CowStr>;

#[derive(Debug)]
pub enum TranscodeError<L> {
    ConversionError { line_no: u32, error: L },
    IoError(io::Error),
}

impl<L> TranscodeError<L> {
    pub fn into_conversion_error(self) -> Option<(u32, L)> {
        match self {
            TranscodeError::ConversionError { line_no, error } => Some((line_no, error)),
            TranscodeError::IoError(_) => None,
        }
    }

    pub fn into_io_error(self) -> Option<io::Error> {
        match self {
            TranscodeError::ConversionError { .. } => None,
            TranscodeError::IoError(error) => Some(error),
        }
    }
}

impl<L: Into<Box<dyn Error>>> TranscodeError<L> {
    pub fn into_dynamic(self) -> TranscodeError<Box<dyn Error>> {
        match self {
            TranscodeError::ConversionError { line_no, error } => TranscodeError::ConversionError {
                line_no,
                error: error.into(),
            },
            TranscodeError::IoError(error) => TranscodeError::IoError(error),
        }
    }
}

impl<L: Display + Debug> Display for TranscodeError<L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscodeError::ConversionError { line_no, error } => {
                write!(f, "conversion error on line {line_no}: {error}")
            }
            TranscodeError::IoError(error) => write!(f, "I/O error: {error:?}"),
        }
    }
}

impl<L: Display + Debug> error::Error for TranscodeError<L> {}

impl<L> From<io::Error> for TranscodeError<L> {
    fn from(error: io::Error) -> Self {
        TranscodeError::IoError(error)
    }
}

pub fn transcode<L>(
    r: &mut impl BufRead,
    w: &mut impl Write,
    mut processor: impl FnMut(u32, &str) -> Result<String, L>,
) -> Result<(), TranscodeError<L>> {
    let mut read_buf = String::new();
    let mut line_no = 1u32;
    loop {
        match r.read_line(&mut read_buf)? {
            0 => return Ok(()),
            size => match processor(line_no, &read_buf[0..size - 1]) {
                Ok(output) => {
                    writeln!(w, "{}", output)?;
                    read_buf.clear();
                }
                Err(error) => return Err(TranscodeError::ConversionError { line_no, error }),
            },
        }
        line_no += 1;
    }
}

#[cfg(test)]
mod tests;
