use super::*;
use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};
use crate::succinct::Stringlike;

#[test]
fn populate_incremental() {
    let mut dict = Dict::default();

    dict.populate(stringify(["uno", "one", "no"]));
    assert_eq!(
        HashMap::from([(String::from("n"), stringify(["no", "uno", "one"]))]),
        dict.entries
    );

    dict.populate(stringify(["Anna"]));
    assert_eq!(
        HashMap::from([(String::from("n"), stringify(["no", "uno", "one"])),]),
        dict.entries
    );

    dict.populate(stringify(["anna"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "uno", "one"])),
            (String::from("nn"), stringify(["anna"]))
        ]),
        dict.entries
    );

    dict.populate(stringify(["half-time"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "uno", "one"])),
            (String::from("nn"), stringify(["anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );

    dict.populate(stringify(["on", "an", "in", "inn"]));
    assert_eq!(
        HashMap::from([
            (
                String::from("n"),
                stringify(["no", "on", "an", "in", "uno", "one"])
            ),
            (String::from("nn"), stringify(["inn", "anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );

    dict.populate(stringify(["i", "aio"]));
    assert_eq!(
        HashMap::from([
            (
                String::from("n"),
                stringify(["no", "on", "an", "in", "uno", "one"])
            ),
            (String::from("nn"), stringify(["inn", "anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );
}

#[test]
fn count() {
    struct Case {
        input: Vec<&'static str>,
        expect: usize,
    }
    for case in vec![
        Case {
            input: vec![""],
            expect: 0,
        },
        Case {
            input: vec!["in"],
            expect: 1,
        },
        Case {
            input: vec!["in", "on"],
            expect: 2,
        },
        Case {
            input: vec!["in", "on", "at"],
            expect: 3,
        },
    ] {
        let dict = Dict::from(case.input.clone());
        assert_eq!(case.expect, dict.count(), "for input {:?}", &case.input);
    }
}

#[test]
fn resolve() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_fingerprint: &'static str,
        input_position: u8,
        expect: Result<Option<&'static str>, WordResolveError>
    }
    for case in vec! [
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 0,
            expect: Ok(Some("in"))
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 1,
            expect: Ok(Some("on"))
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 2,
            expect: Err(WordResolveError::from_borrowed("no dictionary word at position 2 for fingerprint 'n'"))
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "t",
            input_position: 2,
            expect: Ok(None)
        }
    ] {
        let dict = Dict::from(case.input_dict.clone());
        let actual = dict.resolve(case.input_fingerprint, case.input_position).map(|option_of_string_ref| option_of_string_ref.map(String::as_str));
        assert_eq!(case.expect, actual, "for {:?}", case);
    }
}

#[test]
fn position() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_fingerprint: &'static str,
        input_word: &'static str,
        expect: Option<u8>
    }
    for case in vec! [
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "in",
            expect: Some(0)
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "on",
            expect: Some(1)
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "an",
            expect: None
        },
        Case {
            input_dict: vec![],
            input_fingerprint: "n",
            input_word: "an",
            expect: None
        }
    ] {
        let dict = Dict::from(case.input_dict.clone());
        assert_eq!(case.expect, dict.position(case.input_fingerprint, case.input_word), "for {:?}", case)
    }
}

#[test]
fn contains_fingerprint() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_fingerprint: &'static str,
        expect: bool
    }
    for case in vec! [
        Case {
            input_dict: vec!["in"],
            input_fingerprint: "n",
            expect: true
        },
        Case {
            input_dict: vec!["in"],
            input_fingerprint: "t",
            expect: false
        }
    ] {
        let dict = Dict::from(case.input_dict.clone());
        assert_eq!(case.expect, dict.contains_fingerprint(case.input_fingerprint), "for {:?}", case);
    }
}

#[test]
fn populate_should_fill_to_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..255)
        .into_iter()
        .map(|i| format!("test-{}", "a".repeat(i)))
        .collect::<Vec<_>>();
    dict.populate(words);
}

#[test]
#[should_panic(expected = "too many words associated")]
fn populate_should_not_fill_past_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..256)
        .into_iter()
        .map(|i| format!("test-{}", "a".repeat(i)))
        .collect::<Vec<_>>();
    dict.populate(words);
}

#[test]
fn write_and_read_binary_image() {
    let dict = Dict::from(stringify(["in", "on", "at"]));
    let mut cursor = Cursor::new(Vec::new());
    dict.write_to_binary_image(&mut cursor).unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();
    let loaded = Dict::read_from_binary_image(&mut cursor).unwrap();
    assert_eq!(dict.entries, loaded.entries);
}

#[test]
fn read_from_text_file() {
    let text = r#"
      in
      on
      at
      the
      is
      of
      off"#;
    let mut cursor = Cursor::new(text.as_bytes());
    let loaded = Dict::read_from_text_file(&mut cursor).unwrap();
    assert_eq!(Dict::from(stringify(["in", "on", "at", "the", "is", "of", "off"])).entries, loaded.entries);
}

impl Dict {
    pub fn from(line: impl IntoIterator<Item = impl Stringlike>) -> Dict {
        let mut dict = Dict::default();
        dict.populate(line.into_iter().map(Stringlike::into_owned));
        dict
    }
}

fn stringify<const N: usize>(strings: [&str; N]) -> Vec<String> {
    strings.iter().map(ToString::to_string).collect()
}
