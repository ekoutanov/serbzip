pub mod dict;

use std::borrow::Cow;
use dict::Dict;
use crate::codec::CompressionRule::{Conflict, InDict, NoFingerprintInDict, NotInDictWithVowels};

#[derive(Debug, PartialEq)]
struct Reduction {
    fingerprint: String,
    leading_capital: bool,
    trailing_capitals: u8,
}

impl Reduction {
    fn new(fingerprint: String, leading_capital: bool, trailing_capitals: u8) -> Self {
        Reduction { fingerprint, leading_capital, trailing_capitals }
    }

    fn is_lowercase(&self) -> bool {
        !self.leading_capital && self.trailing_capitals == 0
    }

    fn take_if_lowercase(self) -> Option<Self> {
        if self.is_lowercase() {
            Some(self)
        } else {
            None
        }
    }
}

impl From<&str> for Reduction {
    fn from(word: &str) -> Self {
        let mut fingerprint = String::new();
        let mut leading_capital = false;
        let mut trailing_capitals = 0;
        for (position, ch) in word.chars().enumerate() {
            if ch.is_uppercase() {
                match position {
                    0 => leading_capital = true,
                    _ => trailing_capitals += 1,
                }

                if !is_vowel(ch) {
                    fingerprint.push(ch.to_lowercase().next().unwrap())
                }
            } else if !is_vowel(ch) {
                fingerprint.push(ch);
            }
        }
        Reduction::new(fingerprint, leading_capital, trailing_capitals)
    }
}

fn is_vowel(ch: char) -> bool {
    match ch {
        'a' | 'A' | 'e' | 'E' | 'i' | 'I' | 'o' | 'O' | 'u' | 'U' => true,
        'а' | 'А' | 'э' | 'Э' | 'ы' | 'Ы' | 'у' | 'У' | 'я' | 'Я' => true,
        'е' | 'Е' | 'ё' | 'Ё' | 'ю' | 'Ю' | 'и' | 'И' | 'о' | 'О' => true,
        _ => false,
    }
}

trait Fragment {
    fn append_to(&self, buf: &mut String);
}

#[derive(Debug, PartialEq)]
struct EncodedWord {
    leading_spaces: u8,
    body: String,
}

impl EncodedWord {
    fn new(leading_spaces: u8, body: String) -> Self {
        assert!(body.len() > 0);
        EncodedWord { leading_spaces, body }
    }

    fn parse_line(line: &str) -> Vec<EncodedWord> {
        let mut buf = Some(String::new());
        let mut leading_spaces: u8 = 0;
        let chars = line.chars();
        let mut words = Vec::new();
        for ch in chars {
            if ch == ' ' || ch == '\u{200E}' {  // we also support the LRM codepoint
                if !buf.as_ref().unwrap().is_empty() {
                    words.push(EncodedWord {
                        leading_spaces,
                        body: buf.replace(String::new()).unwrap(),
                    });
                    leading_spaces = 0;
                } else {
                    leading_spaces += 1;
                }
            } else {
                buf.as_mut().unwrap().push(ch);
            }
        }

        if !buf.as_ref().unwrap().is_empty() {
            words.push(EncodedWord {
                leading_spaces,
                body: buf.take().unwrap(),
            });
        }
        words
    }
}

impl Fragment for EncodedWord {
    fn append_to(&self, buf: &mut String) {
        for _ in 0..self.leading_spaces {
            buf.push(' ');
        }
        buf.push_str(&self.body);
    }
}

#[derive(Debug, PartialEq)]
struct SplitWord<'a> {
    prefix: Cow<'a, str>,
    suffix: Cow<'a, str>,
}

impl SplitWord<'_> {
    fn from(word: &str) -> SplitWord {
        let position = word.chars().enumerate().position(|(position, ch)| {
            match position {
                0 => !(ch.is_alphabetic() || ch == '\\'), // allow the escape character to be the first in the string
                _ => !ch.is_alphabetic()                  // otherwise, split on non-alphabetic characters
            }
        });
        match position {
            None => SplitWord {
                prefix: Cow::Borrowed(word),
                suffix: Cow::Borrowed(""),
            },
            Some(position) => {
                let prefix = String::from_iter(word.chars().take(position));
                let suffix = String::from_iter(word.chars().skip(position));
                SplitWord { prefix: Cow::Owned(prefix), suffix: Cow::Owned(suffix) }
            },
        }
        // let (prefix, suffix): (Vec<_>, Vec<_>) = word.chars().enumerate().partition(|&(position, ch)| {
        //     match position {
        //         0 => ch.is_alphabetic() || ch == '\\', // allow the escape character to be the first in the string
        //         _ => ch.is_alphabetic()                // otherwise, split on non-alphabetic characters
        //     }
        // });
        // let prefix = String::from_iter(prefix.iter().map(|(_, ch)| ch));
        // let suffix = String::from_iter(suffix.iter().map(|(_, ch)| ch));
        // SplitWord { prefix, suffix }
    }
}

pub fn compress_line(dict: &Dict, line: &str) -> String {
    let mut buf = String::new();
    // let words = Word::parse_line(line);
    let words = line.split_whitespace();
    // println!("words: {words:?}");
    for (index, word) in words.enumerate() {
        if index > 0 {
            buf.push(' ');
        }
        let compressed_word = compress_word(dict, word);
        compressed_word.append_to(&mut buf);
    }
    buf
}

#[derive(Debug)]
enum CompressionRule {
    InDict,
    NotInDictWithVowels,
    NoFingerprintInDict,
    Conflict
}

fn compress_word(dict: &Dict, word: &str) -> EncodedWord {
    let split = SplitWord::from(word);
    let prefix_reduction = Reduction::from(&split.prefix as &str);
    // println!("prefix reduction {prefix_reduction:?}");
    let lowercase_prefix = split.prefix.to_lowercase();
    let (encoded_prefix, rule) = match dict.position(&prefix_reduction.fingerprint, &lowercase_prefix) {
        None => {
            if split.prefix.len() != prefix_reduction.fingerprint.len() {
                // the input comprises one or more vowels
                ((0, lowercase_prefix), NotInDictWithVowels)
            } else if !dict.contains_fingerprint(&prefix_reduction.fingerprint) {
                // the input comprises only consonants and its fingerprint is not in the dict
                ((0, lowercase_prefix), NoFingerprintInDict)
            } else {
                // the input comprises only consonants and there are other words in the
                // dict with a matching fingerprint
                ((0, format!("\\{}", split.prefix)), Conflict)
            }
        }
        Some(position) => {
            // the dictionary contains the lower-cased input
            ((position, prefix_reduction.fingerprint), InDict)
        }
    };

    // println!("rule: {rule:?}");
    match rule {
        Conflict => EncodedWord::new(encoded_prefix.0, encoded_prefix.1 + &split.suffix),
        _ => {
            let recapitalised_prefix = restore_capitalisation(
                encoded_prefix.1,
                prefix_reduction.leading_capital,
                prefix_reduction.trailing_capitals != 0,
            );
            EncodedWord::new(encoded_prefix.0, recapitalised_prefix + &split.suffix)
        }
    }
}

fn restore_capitalisation(
    lowercase_word: String,
    leading_capital: bool,
    nonleading_capital: bool,
) -> String {
    match lowercase_word.len() {
        1 => {
            if leading_capital {
                lowercase_word.to_uppercase()
            } else {
                lowercase_word
            }
        }
        _ => {
            if leading_capital && nonleading_capital {
                lowercase_word.to_uppercase()
            } else if leading_capital {
                let mut chars = lowercase_word.chars();
                chars.next().unwrap().to_uppercase().to_string() + chars.as_str()
            } else {
                lowercase_word
            }
        }
    }
}

pub fn expand_line(dict: &Dict, line: &str) -> Result<String, String> {
    let mut buf = String::new();
    let words = EncodedWord::parse_line(line);
    // println!("words: {words:?}");
    for (index, word) in words.into_iter().enumerate() {
        if index > 0 {
            buf.push(' ');
        }
        let expanded_word = expand_word(dict, word)?;
        buf.push_str(&expanded_word);
    }
    Ok(buf)
}

const ESCAPE: u8 = '\\' as u8;

fn expand_word(dict: &Dict, word: EncodedWord) -> Result<String, String> {
    let split = SplitWord::from(&word.body);
    if split.prefix.is_empty() {
        return Ok(word.body)
    }

    let recapitalised_prefix = if split.prefix.as_bytes()[0] == ESCAPE {
        // escaped word
        if split.prefix.len() > 1 {
            split.prefix[1..split.prefix.len()].to_owned()
        } else {
            split.prefix.into_owned()
        }
    } else {
        let mut chars = split.prefix.chars();
        let leading_capital = chars.next().unwrap().is_uppercase();
        let nonleading_capital = chars.next().map_or(false, char::is_uppercase);

        let resolved_lowercase = if contains_vowels(&split.prefix) {
            // word encoded with vowels
            split.prefix.into_owned()
        } else {
            let lowercase_word = split.prefix.to_lowercase();
            match dict.resolve(&lowercase_word, word.leading_spaces)? {
                None => {
                    // the fingerprint is not in the dictionary
                    lowercase_word
                }
                Some(resolved) => {
                    // resolved a word from the dictionary
                    resolved.clone()
                }
            }
        };

        restore_capitalisation(resolved_lowercase, leading_capital, nonleading_capital)
    };

    Ok(recapitalised_prefix + &split.suffix)
}

fn contains_vowels(text: &str) -> bool {
    text.chars().any(is_vowel)
}

#[cfg(test)]
mod tests;
