pub mod dict;

use dict::Dict;

#[derive(Debug, PartialEq)]
struct Word(u8, String);

#[derive(Debug, PartialEq)]
struct Reduction {
    reduced: String,
    leading_capital: bool,
    trailing_capitals: u8,
}

impl Reduction {
    fn is_lowercase(&self) -> bool {
        !self.leading_capital && self.trailing_capitals == 0
    }

    fn take_if_lowercase(self) -> Option<Self> {
        if self.is_lowercase() { Some(self) } else { None }
    }
}

impl From<&str> for Reduction {
    fn from(word: &str) -> Self {
        let mut reduced = String::new();
        let mut leading_capital = false;
        let mut trailing_capitals = 0;
        for (position, ch) in word.chars().enumerate() {
            if ch.is_uppercase() {
                match position {
                    0 => leading_capital = true,
                    _ => trailing_capitals += 1
                }

                if !is_vowel(ch) {
                    reduced.push(ch.to_lowercase().next().unwrap())
                }
            } else if !is_vowel(ch) {
                reduced.push(ch);
            }
        }
        return Self { reduced, leading_capital, trailing_capitals }
    }
}

fn is_vowel(ch: char) -> bool {
    match ch {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false
    }
}

impl Word {
    fn parse_line(line: &str) -> Vec<Word> {
        let mut buf = Some(String::new());
        let mut leading_chars: u8 = 0;
        let chars = line.chars();
        let mut words = Vec::new();
        for ch in chars {
            if ch == ' ' {
                if !buf.as_ref().unwrap().is_empty() {
                    words.push(Word(leading_chars, buf.replace(String::new()).unwrap()));
                    leading_chars = 0;
                } else {
                    leading_chars += 1;
                }
            } else {
                buf.as_mut().unwrap().push(ch);
            }
        }

        if !buf.as_ref().unwrap().is_empty() {
            words.push(Word(leading_chars, buf.take().unwrap()));
        }
        words
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
        for _ in 0..compressed_word.0 {
            buf.push(' ');
        }
        buf.push_str(&compressed_word.1);
    }
    buf
}

fn compress_word(dict: &Dict, word: &str) -> Word {
    let reduction = Reduction::from(word);
    let lowercase_word = word.to_lowercase();
    let word = match dict.position(&reduction.reduced, &lowercase_word) {
        None => {
            if reduction.is_lowercase() {
                if word.len() != reduction.reduced.len() {
                    // the input comprises one or more vowels
                    Word(0, lowercase_word)
                } else if !dict.contains_key(word) {
                    // the input comprises only consonants and its fingerprint is not in the dict
                    Word(0, lowercase_word)
                } else {
                    // the input comprises only consonants and there are other words in the
                    // dict with a matching fingerprint
                    Word(0, format!("\\{}", lowercase_word))
                }
            } else {
                Word(0, lowercase_word)
            }
        }
        Some(position) => {
            // the dictionary contains the lower-cased input
            Word(position, reduction.reduced)
        }
    };
    Word(word.0, restore_capitalisation(word.1, reduction.leading_capital, reduction.trailing_capitals))
}

fn restore_capitalisation(lowercase_word: String, leading_capital: bool, trailing_capitals: u8) -> String {
    match lowercase_word.len() {
        1 => {
            if leading_capital {
                lowercase_word.to_uppercase()
            } else {
                lowercase_word
            }
        }
        _ => {
            if leading_capital && trailing_capitals > 0 {
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

pub fn expand_line(dict: &Dict, line: &str) -> String {
    let mut buf = String::new();
    let words = Word::parse_line(line);
    // println!("words: {words:?}");
    for (index, word) in words.into_iter().enumerate() {
        if index > 0 {
            buf.push(' ');
        }
        let expanded_word = expand_word(dict, word);
        buf.push_str(&expanded_word);
    }
    buf
}

const ESCAPE: u8 = '\\' as u8;

fn expand_word(dict: &Dict, word: Word) -> String {
    // let chars = word.1.chars();
    if word.1.as_bytes()[0] == ESCAPE {
        // escaped word
        word.1
    } else if contains_vowels(&word.1) {
        // word encoded with vowels
        word.1
    } else {
        let reduction = Reduction::from(&word.1 as &str);
        match dict.resolve(&reduction.reduced, word.0) {
            None => {
                // the fingerprint is not in the dictionary
                word.1
            }
            Some(resolved) => {
                // resolved a word from the dictionary
                resolved.clone()
            }
        }
        // let lowercase = if reduction.is_lowercase() { word.1 } else { word.1.to_lowercase() };
        // match dict.position(&reduction.reduced, &lowercase) {
        //     None => {
        //         // the fingerprint is not in the dictionary
        //         lowercase
        //     }
        //     Some(position) => {
        //
        //     }
        // }
    }
}

fn contains_vowels(text: &str) -> bool {
    text.chars().any(is_vowel)
}

#[cfg(test)]
mod tests;