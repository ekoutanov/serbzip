use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use bincode::config;
use bincode::error::{DecodeError, EncodeError};
use crate::codecs::balkanoid::Reduction;

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct Dict {
    entries: HashMap<String, Vec<String>>
}

impl Default for Dict {
    fn default() -> Self {
        Self { entries: HashMap::new() }
    }
}

impl Dict {
    pub fn populate(&mut self, line: impl IntoIterator<Item = String>) {
        for word in line {
            let reduction = Reduction::from(&word as &str).take_if_lowercase();
            if let Some(Reduction { fingerprint, .. }) = reduction {
                if !fingerprint.is_empty() {
                    let mapped_words = match self.entries.entry(fingerprint) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(vec![])
                    };
                    if mapped_words.len() == u8::MAX as usize {
                        panic!("too many words associated with the fingerprint '{}'", word);
                    }
                    mapped_words.push(word);
                    mapped_words.sort_by(|lhs, rhs| lhs.len().cmp(&rhs.len()));
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.entries.values().map(|values|values.len()).sum()
    }

    pub(crate) fn resolve(&self, fingerprint: &str, position: u8) -> Result<Option<&String>, String> {
        match self.entries.get(fingerprint) {
            None => Ok(None),
            Some(entry) => {
                match entry.get(position as usize) {
                    None => Err(format!("no dictionary word at position {position} for fingerprint '{fingerprint}'")),
                    Some(word) => Ok(Some(word))
                }
            }
        }
    }

    pub(crate) fn position(&self, fingerprint: &str, word: &str) -> Option<u8> {
        match self.entries.get(fingerprint) {
            None => None,
            Some(entry) => entry.iter().position(|existing| existing == word).map(|pos| pos as u8)
        }
    }

    pub(crate) fn contains_fingerprint(&self, fingerprint: &str) -> bool {
        self.entries.contains_key(fingerprint)
    }

    pub fn write_to_binary_image(&self, w: &mut impl Write) -> Result<usize, EncodeError> {
        bincode::encode_into_std_write(self, w, config::standard())
    }

    pub fn read_from_binary_image(r: &mut impl Read) -> Result<Dict, DecodeError> {
        bincode::decode_from_std_read(r, config::standard())
    }

    pub fn read_from_text_file(r: &mut impl Read) -> Result<Dict, io::Error> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        let mut dict = Dict::default();
        for line in buf.lines() {
            let line = line.split_whitespace();
            dict.populate(line.map(ToOwned::to_owned));
        }
        Ok(dict)
    }
}

#[cfg(test)]
mod tests;