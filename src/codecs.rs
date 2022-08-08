use std::convert::Infallible;
use crate::transcoder;
use crate::transcoder::TranscodeError;
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::io::Write;

pub mod armenoid;
pub mod balkanoid;

pub trait Codec {
    type ExpandError: Error;

    fn compress_line(&self, line: &str) -> String;

    fn expand_line(&self, line: &str) -> Result<String, Self::ExpandError>;

    fn compress(&self, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), io::Error> {
        let result: Result<(), TranscodeError<Infallible>> =
            transcoder::transcode(r, w, |_, line| Ok(self.compress_line(line)));
        result.map_err(|err| err.into_io_error().unwrap())
    }

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
