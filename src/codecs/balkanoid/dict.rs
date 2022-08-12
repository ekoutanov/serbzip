//! The Balkanoid dictionary, as well as methods for populating it from
//! an input stream (in text and binary modes) and for writing a binary
//! image to a file.

use std::cmp::Ordering;
use crate::codecs::balkanoid::Reduction;
use crate::succinct::{CowStr, Errorlike};
use bincode::config;
use bincode::error::{DecodeError, EncodeError};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};

/// The dictionary used by [`Balkanoid`](super::Balkanoid).
#[derive(Default, Debug, bincode::Encode, bincode::Decode)]
pub struct Dict {
    entries: HashMap<String, Vec<String>>,
}

pub type WordResolveError = Errorlike<CowStr>;

fn comparator(lhs: &String, rhs: &String) -> Ordering {
    lhs.len().cmp(&rhs.len()).then(lhs.cmp(rhs))
}

impl From<HashMap<String, Vec<String>>> for Dict {
    /// Instantiates a dictionary from a given map.
    fn from(entries: HashMap<String, Vec<String>>) -> Self {
        Self { entries }
    }
}

impl Dict {
    /// Populates the dictionary from a collection of [`String`] items.
    ///
    /// # Panics
    /// If more than [`u8::MAX`] words end up associated with the same fingerprint.
    pub fn populate(&mut self, line: impl IntoIterator<Item = String>) {
        for word in line {
            let reduction = Reduction::from(&word as &str).take_if_lowercase();
            if let Some(Reduction { fingerprint, .. }) = reduction {
                if !fingerprint.is_empty() {
                    let mapped_words = match self.entries.entry(fingerprint) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(vec![]),
                    };
                    assert_ne!(mapped_words.len(), u8::MAX as usize, "too many words associated with the fingerprint '{}'", word);
                    if mapped_words.binary_search_by(|candidate| comparator(candidate, &word)).is_ok() {
                        continue   // don't collate duplicate words
                    }
                    mapped_words.push(word);
                    mapped_words.sort_by(comparator);
                }
            }
        }
    }

    /// Obtains the aggregate count of words collated in this dictionary.
    pub fn count(&self) -> usize {
        self.entries.values().map(Vec::len).sum()
    }

    /// Resolves a collated word, given its fingerprint and position.
    ///
    /// If the fingerprint does not exist, returns `Ok(None)`. Otherwise, if the fingerprint
    /// exists and there is a word at the given position, returns `Ok(Some(..))`, containing
    /// the collated word. Otherwise, if the fingerprint exists, but no word is collated at
    /// the given position, this method returns a [`WordResolveError`].
    ///
    /// # Errors
    /// [`WordResolveError`] if the given fingerprint exists, but the position does not
    /// resolve to a collated word.
    pub(crate) fn resolve(
        &self,
        fingerprint: &str,
        position: u8,
    ) -> Result<Option<&String>, WordResolveError> {
        match self.entries.get(fingerprint) {
            None => Ok(None),
            Some(entry) => match entry.get(position as usize) {
                None => Err(Errorlike::from_owned(format!(
                    "no dictionary word at position {position} for fingerprint '{fingerprint}'"
                ))),
                Some(word) => Ok(Some(word)),
            },
        }
    }

    /// Obtains the position of a word, given its fingerprint.
    ///
    /// The position is returned as a [`Some(u8)`] if the fingerprint exists and the given word
    /// is in the vector. Otherwise, [`None`] is returned.
    pub(crate) fn position(&self, fingerprint: &str, word: &str) -> Option<u8> {
        match self.entries.get(fingerprint) {
            None => None,
            Some(entry) => entry
                .iter()
                .position(|existing| existing == word)
                .map(|pos| u8::try_from(pos).unwrap()),  // pos is guaranteed to be less than 2^8
        }
    }

    /// Checks whether the given fingerprint exists in the dictionary.
    pub(crate) fn contains_fingerprint(&self, fingerprint: &str) -> bool {
        self.entries.contains_key(fingerprint)
    }

    /// Outputs the contents of the dictionary to the given writer, using the `bincode` protocol.
    ///
    /// # Errors
    /// [`EncodeError`] if an error occurred during encoding.
    pub fn write_to_binary_image(&self, w: &mut impl Write) -> Result<usize, EncodeError> {
        bincode::encode_into_std_write(self, w, config::standard())
    }

    /// Loads a new dictionary from a given reader, using the `bincode` protocol.
    ///
    /// # Errors
    /// [`DecodeError`] if an error occurred during decoding.
    pub fn read_from_binary_image(r: &mut impl Read) -> Result<Dict, DecodeError> {
        bincode::decode_from_std_read(r, config::standard())
    }

    /// Loads a new dictionary from a given reader, the latter containing a newline-delimited
    /// wordlist.
    ///
    /// # Errors
    /// [`io::Error`] if an I/O error occurs.
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
pub(in crate::codecs::balkanoid) mod tests;
