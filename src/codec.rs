use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, PartialEq)]
struct Word(u8, String);

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

    fn reduce(lowercase_word: &str) -> String {
        let mut buf = String::new();
        for ch in lowercase_word.chars() {
            if !Self::is_vowel(ch) {
                buf.push(ch);
            }
        }
        return buf
    }

    fn is_vowel(ch: char) -> bool {
        match ch {
            'a' | 'e' | 'i' | 'o' | 'u' => true,
            _ => false
        }
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
        for word in line {
            let word = word.to_lowercase();
            let reduced = Word::reduce(&word);
            let mapped_words = match self.entries.entry(reduced) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(vec![])
            };
            mapped_words.push(word);
            mapped_words.sort_by(|lhs, rhs| lhs.len().cmp(&rhs.len()));
        }
    }
}

pub fn encode_line(dict: &Dict, line: &str) -> String {
    let mut buf = String::new();
    let words = Word::parse_line(line);
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            buf.push(' ');
        }
        let encoded_word = encode_word(dict, word);
        for _ in 0..encoded_word.0 {
            buf.push(' ');
        }
        buf.push_str(&encoded_word.1);
    }
    buf
}

fn encode_word(dict: &Dict, word: &Word) -> Word {
    let encoded = Word::reduce(&word.1.to_lowercase());
    Word(0, encoded)
}

#[cfg(test)]
mod tests;