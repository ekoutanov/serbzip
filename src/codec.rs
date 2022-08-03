use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::Index;

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
            if !is_vowel(ch) {
                reduced.push(ch);
            }

            if ch.is_uppercase() {
                match position {
                    0 => leading_capital = true,
                    _ => trailing_capitals += 1
                }
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

#[derive(Debug)]
pub struct Dict {
    entries: HashMap<String, Vec<String>>
}

impl Default for Dict {
    fn default() -> Self {
        Self { entries: HashMap::new() }
    }
}

impl Dict {
    pub fn populate(&mut self, line: &[String]) {
        for word in line.into_iter() {
            let reduction = Reduction::from(&word as &str).take_if_lowercase();
            if let Some(Reduction { reduced, .. }) = reduction {
                let mapped_words = match self.entries.entry(reduced) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(vec![])
                };
                mapped_words.push(word.to_owned()); //TODO not owned
                mapped_words.sort_by(|lhs, rhs| lhs.len().cmp(&rhs.len()));
            }
        }
    }

    pub fn count(&self) -> usize {
        self.entries.values().map(|values|values.len()).sum()
    }

    fn position(&self, key: &str) -> Option<usize> {
        match self.entries.get(key) {
            None => None,
            Some(entry) => entry.iter().position(|word| word == key)
        }
    }
}

pub fn compress_line(dict: &Dict, line: &str) -> String {
    let mut buf = String::new();
    let words = Word::parse_line(line);
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            buf.push(' ');
        }
        let encoded_word = compress_word(dict, word);
        for _ in 0..encoded_word.0 {
            buf.push(' ');
        }
        buf.push_str(&encoded_word.1);
    }
    buf
}

fn compress_word(dict: &Dict, word: &Word) -> Word {
    let encoded = Reduction::from(&word.1.to_lowercase() as &str).reduced;
    Word(0, encoded)
}

#[cfg(test)]
mod tests;