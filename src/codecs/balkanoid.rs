pub mod dict;

use crate::codecs::balkanoid::dict::WordResolveError;
use crate::codecs::Codec;
use crate::succinct::Stringlike;
pub use dict::Dict;
use std::borrow::Cow;

pub struct Balkanoid<'a> {
    dict: &'a Dict,
}

impl<'a> Balkanoid<'a> {
    pub fn new(dict: &'a Dict) -> Self {
        Self { dict }
    }
}

impl Codec for Balkanoid<'_> {
    type ExpandError = WordResolveError;

    fn compress_line(&self, line: &str) -> String {
        let mut buf = String::new();
        // let words = Word::parse_line(line);
        let words = line.split_whitespace();
        // println!("words: {words:?}");
        for (index, word) in words.enumerate() {
            if index > 0 {
                buf.push(' ');
            }
            let compressed_word = compress_word(self.dict, word);
            for _ in 0..compressed_word.leading_spaces {
                buf.push(' ');
            }
            buf.push_str(&compressed_word.body);
        }
        buf
    }

    fn expand_line(&self, line: &str) -> Result<String, Self::ExpandError> {
        let mut buf = String::new();
        let words = EncodedWord::parse_line(line);
        // println!("words: {words:?}");
        for (index, word) in words.into_iter().enumerate() {
            if index > 0 {
                buf.push(' ');
            }
            let expanded_word = expand_word(self.dict, word)?;
            buf.push_str(&expanded_word);
        }
        Ok(buf)
    }
}

#[derive(Debug, PartialEq)]
struct Reduction {
    fingerprint: String,
    leading_capital: bool,
    trailing_capitals: u8,
}

impl Reduction {
    fn new(fingerprint: String, leading_capital: bool, trailing_capitals: u8) -> Self {
        Reduction {
            fingerprint,
            leading_capital,
            trailing_capitals,
        }
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
                    fingerprint.push(ch.to_lowercase().next().unwrap());
                }
            } else if !is_vowel(ch) {
                fingerprint.push(ch);
            }
        }
        Reduction::new(fingerprint, leading_capital, trailing_capitals)
    }
}

fn is_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'A'
            | 'e'
            | 'E'
            | 'i'
            | 'I'
            | 'o'
            | 'O'
            | 'u'
            | 'U'
            | 'а'
            | 'А'
            | 'э'
            | 'Э'
            | 'ы'
            | 'Ы'
            | 'у'
            | 'У'
            | 'я'
            | 'Я'
            | 'е'
            | 'Е'
            | 'ё'
            | 'Ё'
            | 'ю'
            | 'Ю'
            | 'и'
            | 'И'
            | 'о'
            | 'О'
    )
}

#[derive(Debug, PartialEq)]
struct EncodedWord {
    leading_spaces: u8,
    body: String,
}

impl EncodedWord {
    fn new(leading_spaces: u8, body: String) -> Self {
        assert!(!body.is_empty());
        EncodedWord {
            leading_spaces,
            body,
        }
    }

    fn parse_line(line: &str) -> Vec<EncodedWord> {
        let mut buf = Some(String::new());
        let mut leading_spaces: u8 = 0;
        let chars = line.chars();
        let mut words = Vec::new();
        for ch in chars {
            if ch == ' ' || ch == '\u{200E}' {
                // we also support the LRM codepoint
                if buf.as_ref().unwrap().is_empty() {
                    leading_spaces += 1;
                } else {
                    words.push(EncodedWord {
                        leading_spaces,
                        body: buf.replace(String::new()).unwrap(),
                    });
                    leading_spaces = 0;
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

#[derive(Debug, PartialEq)]
struct PunctuatedWord<'a> {
    prefix: Cow<'a, str>,
    suffix: Cow<'a, str>,
}

impl PunctuatedWord<'_> {
    fn from(word: &str) -> PunctuatedWord {
        let position = word.chars().enumerate().position(|(position, ch)| {
            // println!("position: {position}, char: {ch}");
            match position {
                0 => !ch.is_alphabetic() && ch != '\\', // allow the escape character to be the first in the string
                _ => !ch.is_alphabetic(), // otherwise, split on non-alphabetic characters
            }
        });
        // println!("got position: {position:?}");
        match position {
            None => PunctuatedWord {
                prefix: Cow::Borrowed(word),
                suffix: Cow::Borrowed(""),
            },
            Some(position) => {
                let prefix = word.chars().take(position).collect::<String>();
                let suffix = word.chars().skip(position).collect::<String>();
                PunctuatedWord {
                    prefix: Cow::Owned(prefix),
                    suffix: Cow::Owned(suffix),
                }
            }
        }
    }
}

#[derive(Debug)]
enum CompactionRule {
    InDict,
    NotInDictWithVowels,
    NoFingerprintInDict,
    Conflict,
    LeadingEscape,
}

fn compress_word(dict: &Dict, word: &str) -> EncodedWord {
    assert!(!word.is_empty());
    let punctuated = PunctuatedWord::from(word);

    let (encoded_prefix, _) = {
        let first_char = punctuated.prefix.chars().next();
        match first_char {
            Some('\\') => {
                // the first character marks the start of an escape sequence
                (
                    (0, format!("\\{}", punctuated.prefix)),
                    CompactionRule::LeadingEscape,
                )
            }
            _ => {
                // println!("punctuated: {punctuated:?}");
                let prefix_reduction = Reduction::from(&punctuated.prefix as &str);
                // println!("prefix reduction {prefix_reduction:?}");
                let lowercase_prefix = punctuated.prefix.to_lowercase();
                match dict.position(&prefix_reduction.fingerprint, &lowercase_prefix) {
                    None => {
                        if punctuated.prefix.len() != prefix_reduction.fingerprint.len() {
                            // the input comprises one or more vowels
                            (
                                (0, punctuated.prefix.into_owned()),
                                CompactionRule::NotInDictWithVowels,
                            )
                        } else if !dict.contains_fingerprint(&prefix_reduction.fingerprint) {
                            // the input comprises only consonants and its fingerprint is not in the dict
                            (
                                (0, punctuated.prefix.into_owned()),
                                CompactionRule::NoFingerprintInDict,
                            )
                        } else {
                            // the input comprises only consonants and there are other words in the
                            // dict with a matching fingerprint
                            (
                                (0, format!("\\{}", punctuated.prefix)),
                                CompactionRule::Conflict,
                            )
                        }
                    }
                    Some(position) => {
                        // the dictionary contains the lower-cased input
                        let recapitalised_prefix = restore_capitalisation(
                            prefix_reduction.fingerprint,
                            prefix_reduction.leading_capital,
                            prefix_reduction.trailing_capitals != 0,
                        );
                        ((position, recapitalised_prefix), CompactionRule::InDict)
                    }
                }
            }
        }
    };
    // println!("rule: {rule:?}");
    EncodedWord::new(encoded_prefix.0, encoded_prefix.1 + &punctuated.suffix)
}

// fn compress_word(dict: &Dict, word: &str) -> EncodedWord {
//     assert!(! word.is_empty());
//     println!("word: '{word}'");
//     let punctuated = PunctuatedWord::from(word);
//
//     if let Some(first_char) = punctuated.prefix.chars().next() {
//         if first_char == '\\' {
//             return EncodedWord::new(0, format!("\\{}{}", punctuated.prefix, punctuated.suffix));
//         }
//     }
//
//     println!("punctuated: {punctuated:?}");
//     let prefix_reduction = Reduction::from(&punctuated.prefix as &str);
//     println!("prefix reduction {prefix_reduction:?}");
//     let lowercase_prefix = punctuated.prefix.to_lowercase();
//     let (encoded_prefix, rule) =
//         match dict.position(&prefix_reduction.fingerprint, &lowercase_prefix) {
//             None => {
//                 if punctuated.prefix.len() != prefix_reduction.fingerprint.len() {
//                     // the input comprises one or more vowels
//                     ((0, punctuated.prefix.into_owned()), CompactionRule::NotInDictWithVowels)
//                 } else if !dict.contains_fingerprint(&prefix_reduction.fingerprint) {
//                     // the input comprises only consonants and its fingerprint is not in the dict
//                     ((0, punctuated.prefix.into_owned()), CompactionRule::NoFingerprintInDict)
//                 } else {
//                     // the input comprises only consonants and there are other words in the
//                     // dict with a matching fingerprint
//                     (
//                         (0, format!("\\{}", punctuated.prefix)),
//                         CompactionRule::Conflict,
//                     )
//                 }
//             }
//             Some(position) => {
//                 // the dictionary contains the lower-cased input
//                 (
//                     (position, prefix_reduction.fingerprint),
//                     CompactionRule::InDict,
//                 )
//             }
//         };
//
//     println!("rule: {rule:?}");
//     match rule {
//         CompactionRule::InDict => {
//             let recapitalised_prefix = restore_capitalisation(
//                 encoded_prefix.1,
//                 prefix_reduction.leading_capital,
//                 prefix_reduction.trailing_capitals != 0,
//             );
//             EncodedWord::new(encoded_prefix.0, recapitalised_prefix + &punctuated.suffix)
//         }
//         _ => {
//             // println!("encoded: {encoded_prefix:?}");
//             EncodedWord::new(encoded_prefix.0, encoded_prefix.1 + &punctuated.suffix)
//         }
//     }
// }

fn restore_capitalisation(
    lowercase_word: String,
    leading_capital: bool,
    nonleading_capital: bool,
) -> String {
    if nonleading_capital {
        lowercase_word.to_uppercase()
    } else if leading_capital {
        let mut chars = lowercase_word.chars();
        chars.next().unwrap().to_uppercase().to_string() + chars.as_str()
    } else {
        lowercase_word
    }
}

const ESCAPE: u8 = b'\\';

fn expand_word(dict: &Dict, word: EncodedWord) -> Result<String, WordResolveError> {
    let punctuated = PunctuatedWord::from(&word.body);
    if punctuated.prefix.is_empty() {
        return Ok(word.body);
    }

    let recapitalised_prefix = if punctuated.prefix.as_bytes()[0] == ESCAPE {
        // escaped word
        // if punctuated.prefix.len() > 1 {
        punctuated.prefix[1..punctuated.prefix.len()].into_owned()
        // } else {
        //     punctuated.prefix.into_owned()
        // }
    } else {
        let mut chars = punctuated.prefix.chars();
        let leading_capital = chars.next().unwrap().is_uppercase();
        let nonleading_capital = chars.next().map_or(false, char::is_uppercase);

        if contains_vowels(&punctuated.prefix) {
            // word encoded with vowels
            punctuated.prefix.into_owned()
        } else {
            let lowercase_word = punctuated.prefix.to_lowercase();
            match dict.resolve(&lowercase_word, word.leading_spaces)? {
                None => {
                    // the fingerprint is not in the dictionary
                    punctuated.prefix.into_owned()
                }
                Some(resolved) => {
                    // resolved a word from the dictionary
                    restore_capitalisation(resolved.clone(), leading_capital, nonleading_capital)
                }
            }
        }
    };

    Ok(recapitalised_prefix + &punctuated.suffix)
}

fn contains_vowels(text: &str) -> bool {
    text.chars().any(is_vowel)
}

#[cfg(test)]
mod tests;
