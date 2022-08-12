//! Machinery common to all codec implementations.

use crate::transcoder;
use crate::transcoder::TranscodeError;
use std::convert::Infallible;
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::io::Write;

pub mod armenoid;
pub mod balkanoid;

/// The specification of a codec.
///
/// Codecs work line by line. As such, they require two method implementations:
/// [`compress_line`](Codec::compress_line) — to encode one line of input text, and
/// [`expand_line`](Codec::expand_line) — to perform the reverse operation.
///
/// Compression is not allowed to return an error; the thinking is that all
/// text should be compressible. Expansion, however, may return an error. It is possible
/// that the output of compression may have been mangled with, in which case expansion
/// is not possible.
pub trait Codec {
    /// The type of error returned during expansion.
    type ExpandError: Error;

    /// Compresses a line of text, returning its encoded representation as a [`String`].
    fn compress_line(&self, line: &str) -> String;

    /// Expands a line of text, returning its decoded representation as a [`String`].
    ///
    /// # Errors
    /// Expansion is not always possible, in which case an [`ExpandError`](Self::ExpandError)
    /// is returned.
    fn expand_line(&self, line: &str) -> Result<String, Self::ExpandError>;

    /// A helper method for compressing a series of lines from a given buffered reader,
    /// outputting the result into the given writer.
    ///
    /// # Errors
    /// [`io::Error`] if the reader or the writer encountered an I/O error.
    fn compress(&self, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), io::Error> {
        let result: Result<(), TranscodeError<Infallible>> =
            transcoder::transcode(r, w, |_, line| Ok(self.compress_line(line)));
        result.map_err(|err| err.into_io_error().unwrap())
    }

    /// A helper method for expanding a series of lines from a given buffered reader,
    /// outputting the result into the given writer.
    ///
    /// # Errors
    /// [`TranscodeError`] if the reader or the writer encountered an I/O error, or
    /// if the codec was unable to expand a line.
    fn expand(
        &self,
        r: &mut impl BufRead,
        w: &mut impl Write,
    ) -> Result<(), TranscodeError<Self::ExpandError>> {
        transcoder::transcode(r, w, |_, line| self.expand_line(line))
    }
}

#[cfg(test)]
mod tests;
