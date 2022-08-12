//! Houses reusable machinery for transcoding streams, line by line.

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, Write};
use std::{error, io};

/// Errors resulting from a transcoding operation.
#[derive(Debug)]
pub enum TranscodeError<L> {
    /// When an error occurred while processing a particular line.
    ConversionError { line_no: u32, error: L },

    // When the error relates to I/O.
    IoError(io::Error),
}

impl<L> TranscodeError<L> {
    /// Maps the error into a [`TranscodeError::ConversionError`].
    pub fn into_conversion_error(self) -> Option<(u32, L)> {
        match self {
            TranscodeError::ConversionError { line_no, error } => Some((line_no, error)),
            TranscodeError::IoError(_) => None,
        }
    }

    /// Maps the error into a [`TranscodeError::IoError`].
    pub fn into_io_error(self) -> Option<io::Error> {
        match self {
            TranscodeError::ConversionError { .. } => None,
            TranscodeError::IoError(error) => Some(error),
        }
    }
}

impl<L: Into<Box<dyn Error>>> TranscodeError<L> {
    /// Boxes the given error if it is a [`TranscodeError::ConversionError`], returning the line error as an [`Error`] trait.
    ///
    /// This is useful for working with methods that return a generic [`Result<_, Box<dyn Error>>`].
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

/// Transcodes between a buffered reader and a writer, using a given `processor` closure to
/// perform the line-by-line translation.
///
/// This function does not require that the output writer be buffered, although a buffered
/// writer may be supplied for performance.
///
/// # Errors
/// [`TranscodeError<L>`] is returned if either the reader/writer produced an I/O error, or
/// the processor returned an error. In the latter, the error encompasses the line number.
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
