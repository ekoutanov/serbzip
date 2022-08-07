use std::io::Write;
use std::io::BufRead;
use crate::transcoder;
use crate::transcoder::TranscodeError;

pub mod balkanoid;

pub trait Codec {
    fn compress_line(&self, line: &str) -> String;

    fn expand_line(&self, line: &str) -> Result<String, String>;

    fn compress(&self, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), TranscodeError> {
        transcoder::transcode(r, w, |_, line| Ok(self.compress_line(line)))
    }

    fn expand(&self, r: &mut impl BufRead, w: &mut impl Write) -> Result<(), TranscodeError> {
        transcoder::transcode(r, w, |_, line| self.expand_line(line))
    }
}
