use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Read, Write};
use bincode::config;
use bincode::de::read::Reader;

use bincode::error::{DecodeError, EncodeError};

use crate::codec::Reduction;

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
    // pub fn populate(&mut self, line: &[String]) {
    //     for word in line.into_iter() {
    //         let reduction = Reduction::from(&word as &str).take_if_lowercase();
    //         if let Some(Reduction { reduced, .. }) = reduction {
    //             let mapped_words = match self.entries.entry(reduced) {
    //                 Entry::Occupied(entry) => entry.into_mut(),
    //                 Entry::Vacant(entry) => entry.insert(vec![])
    //             };
    //             mapped_words.push(word.to_owned()); //TODO not owned
    //             mapped_words.sort_by(|lhs, rhs| lhs.len().cmp(&rhs.len()));
    //         }
    //     }
    // }

    pub fn populate(&mut self, line: impl IntoIterator<Item = String>) {
        for word in line {
            let reduction = Reduction::from(&word as &str).take_if_lowercase();
            if let Some(Reduction { reduced, .. }) = reduction {
                let mapped_words = match self.entries.entry(reduced) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(vec![])
                };
                if mapped_words.len() == u8::MAX as usize {
                    panic!("too many word associated with the fingerprint '{}'", word);
                }
                mapped_words.push(word);
                mapped_words.sort_by(|lhs, rhs| lhs.len().cmp(&rhs.len()));
            }
        }
    }

    pub fn count(&self) -> usize {
        self.entries.values().map(|values|values.len()).sum()
    }

    pub(crate) fn position(&self, key: &str, word: &str) -> Option<u8> {
        match self.entries.get(key) {
            None => None,
            Some(entry) => entry.iter().position(|existing| existing == word).map(|pos| pos as u8)
        }
    }

    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn write_to_binary_image(&self, w: &mut impl Write) -> Result<usize, EncodeError> {
        bincode::encode_into_std_write(self, w, config::standard())
    }

    pub fn load_from_binary_image(r: &mut impl Reader) -> Result<Dict, DecodeError> {
        bincode::decode_from_reader(r, config::standard())
    }
}

#[cfg(test)]
mod tests;