//! The Balkanoid dictionary, as well as methods for populating it from
//! an input stream (in text and binary modes) and for writing a binary
//! image to a file.

use crate::codecs::balkanoid::Reduction;
use crate::succinct::{CowStr, Errorlike};
use bincode::config;
pub use bincode::error::{DecodeError, EncodeError};
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};

/// Emitted when a word vector is about to overflow; i.e., an attempt was made to add more than 2^8 entries.
pub type OverflowError = Errorlike<CowStr>;

/// A vector of words that is capped in size to 2^8 entries, is always sorted and contains
/// no duplicate entries.
#[derive(Default, Debug, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct WordVec(Vec<String>);

impl WordVec {
    /// Creates a new vector from a given iterable of string-like words.
    ///
    /// # Errors
    /// [`OverflowError`] if this operation would result in a vector that exceeds 2^8 entries.
    pub fn new(
        words: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<WordVec, OverflowError> {
        let mut vec = WordVec::default();
        for item in words {
            vec.push(item)?;
        }
        Ok(vec)
    }

    /// Appends a string-like word to the vector.
    ///
    /// # Errors
    /// [`OverflowError`] if this operation would result in a vector that exceeds 2^8 entries.
    pub fn push(&mut self, word: impl Into<String>) -> Result<(), OverflowError> {
        if self.0.len() == u8::MAX as usize {
            Err(OverflowError::borrowed("too many words in vector"))
        } else {
            let word = word.into();
            let position =  self
                .0
                .binary_search_by(|existing| comparator(existing, &word));
            match position {
                Ok(_) => Ok(()),
                Err(position) => {
                    self.0.insert(position, word);
                    Ok(())
                }
            }
        }
    }
}

impl AsRef<Vec<String>> for WordVec {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl From<WordVec> for Vec<String> {
    fn from(vec: WordVec) -> Self {
        vec.0
    }
}

/// The dictionary used by [`Balkanoid`](super::Balkanoid).
#[derive(Default, Debug, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct Dict {
    entries: HashMap<String, WordVec>,
}

/// Emitted when no word could be resolved at the specified position.
pub type WordResolveError = Errorlike<CowStr>;

fn comparator(lhs: &String, rhs: &String) -> Ordering {
    lhs.len().cmp(&rhs.len()).then(lhs.cmp(rhs))
}

impl From<HashMap<String, WordVec>> for Dict {
    /// Instantiates a dictionary from a given map.
    ///
    /// It assumes that the vectors are pre-sorted and there are no more than
    /// [`u8::MAX`] words mapped from any fingerprint.
    ///
    /// # Examples
    /// ```
    /// use std::collections::HashMap;
    /// use serbzip_core::codecs::balkanoid::Dict;
    /// use serbzip_core::codecs::balkanoid::dict::WordVec;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let dict = Dict::from(HashMap::from(
    ///     [
    ///         (String::from("t"), WordVec::new([String::from("at"), String::from("it"), String::from("tea")])?),
    ///         (String::from("n"), WordVec::new([String::from("in"), String::from("no"), String::from("on")])?)
    ///     ]
    /// ));
    /// assert_eq!(6, dict.count());
    /// # Ok(())
    /// # }
    /// ```
    fn from(entries: HashMap<String, WordVec>) -> Self {
        Self { entries }
    }
}

/// Emitted when [`Dict::read_from_text_file`] encounters an error.
#[derive(Debug)]
pub enum ReadFromTextFileError {
    Io(io::Error),
    DictOverflow(OverflowError),
}

impl From<io::Error> for ReadFromTextFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<OverflowError> for ReadFromTextFileError {
    fn from(error: OverflowError) -> Self {
        Self::DictOverflow(error)
    }
}

impl Dict {
    /// Populates the dictionary from a collection of [`String`] words.
    ///
    /// # Errors
    /// [`OverflowError`] if more than 2^8 words would end up associated with the same fingerprint.
    pub fn populate(
        &mut self,
        words: impl IntoIterator<Item = String>,
    ) -> Result<(), OverflowError> {
        for word in words {
            let reduction = Reduction::from(&word as &str).take_if_lowercase();
            if let Some(Reduction { fingerprint, .. }) = reduction {
                if !fingerprint.is_empty() {
                    let mapped_words = match self.entries.entry(fingerprint) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(WordVec::default()),
                    };
                    mapped_words.push(word)?;
                }
            }
        }
        Ok(())
    }

    /// Obtains the aggregate count of words collated in this dictionary.
    pub fn count(&self) -> usize {
        self.entries.values().map(AsRef::as_ref).map(Vec::len).sum()
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
            Some(entry) => match entry.as_ref().get(position as usize) {
                None => Err(Errorlike::owned(format!(
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
                .as_ref()
                .iter()
                .position(|existing| existing == word)
                .map(|pos| u8::try_from(pos).unwrap()), // pos is guaranteed to be less than 2^8
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
    /// [`ReadFromTextFileError`] if an I/O error occurs or if a vector inside the dictionary would overflow.
    pub fn read_from_text_file(r: &mut impl Read) -> Result<Dict, ReadFromTextFileError> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        let mut dict = Dict::default();
        for line in buf.lines() {
            let line = line.split_whitespace();
            dict.populate(line.map(ToOwned::to_owned))?;
        }
        Ok(dict)
    }
}

#[cfg(test)]
pub(in crate::codecs::balkanoid) mod tests;
