use crate::codecs::{Codec, ExpandError};
use std::ops::Add;

pub struct Armenoid;

impl Default for Armenoid {
    fn default() -> Self {
        Self
    }
}

impl Codec for Armenoid {
    fn compress_line(&self, line: &str) -> String {
        line.split_whitespace()
            .enumerate()
            .map(|(pos, _)| if pos == 0 { "inch" } else { " inch" })
            .fold(String::new(), String::add)
    }

    fn expand_line(&self, line: &str) -> Result<String, ExpandError> {
        Ok(line
            .split_whitespace()
            .enumerate()
            .map(|(pos, _)| if pos == 0 { "what" } else { " what" })
            .fold(String::new(), String::add))
    }
}

#[cfg(test)]
mod tests;
