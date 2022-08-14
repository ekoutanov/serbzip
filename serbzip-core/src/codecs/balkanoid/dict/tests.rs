use super::*;
use std::collections::HashMap;
use std::io::{Cursor, ErrorKind, Seek, SeekFrom};

// $coverage:ignore-start

#[test]
fn wordvec_push_capacity_bounds() {
    let mut vec = WordVec::default();
    for i in (0..255).into_iter() {
        assert_eq!(Ok(()), vec.push(format!("{i}")));
    }

    assert!(vec.push(String::from("overflow")).is_err());
}

#[test]
fn wordvec_sort_and_dedup() {
    let vec = WordVec::new(["in", "on", "on", "inn", "no", "in"]).unwrap();
    assert_eq!(
        vec![
            String::from("in"),
            String::from("no"),
            String::from("on"),
            String::from("inn")
        ],
        <WordVec as Into<Vec<_>>>::into(vec)
    );
}

#[test]
fn wordvec_implements_debug() {
    let vec = WordVec::new(["in", "on"]).unwrap();
    assert_eq!(String::from("WordVec([\"in\", \"on\"])"), format!("{:?}", vec));
}

#[test]
fn wordvec_implements_as_ref() {
    let vec = WordVec::new(["in", "on"]).unwrap();
    assert_eq!([String::from("in"), String::from("on")].as_slice(), vec.as_ref());
}

#[test]
fn from_hashmap() {
    let dict = Dict::from(HashMap::from([(
        String::from("n"),
        WordVec::new([String::from("in"), String::from("no"), String::from("on")]).unwrap(),
    )]));
    assert_eq!(Ok(Some(&String::from("no"))), dict.resolve("n", 1));
}

#[test]
fn populate_incremental() {
    let mut dict = Dict::default();

    dict.populate(stringify(["uno", "no"])).unwrap();
    assert_eq!(
        HashMap::from([
            (String::from("n"), WordVec::new(["no", "uno"]).unwrap())
        ]),
        dict.entries
    );

    dict.populate(stringify(["one", "one", "no"])).unwrap();
    assert_eq!(
        HashMap::from([
            (String::from("n"), WordVec::new(["no", "one", "uno"]).unwrap())
        ]),
        dict.entries
    );

    dict.populate(stringify(["Anna"])).unwrap();
    assert_eq!(
        HashMap::from([
            (String::from("n"), WordVec::new(["no", "one", "uno"]).unwrap()),
        ]),
        dict.entries
    );

    dict.populate(stringify(["anna"])).unwrap();
    assert_eq!(
        HashMap::from([
            (String::from("n"), WordVec::new(["no", "one", "uno"]).unwrap()),
            (String::from("nn"), WordVec::new(["anna"]).unwrap())
        ]),
        dict.entries
    );

    dict.populate(stringify(["half-time"])).unwrap();
    assert_eq!(
        HashMap::from([
            (String::from("n"), WordVec::new(["no", "one", "uno"]).unwrap()),
            (String::from("nn"), WordVec::new(["anna"]).unwrap()),
            (String::from("hlf-tm"), WordVec::new(["half-time"]).unwrap())
        ]),
        dict.entries
    );

    dict.populate(stringify(["on", "an", "in", "inn"])).unwrap();
    assert_eq!(
        HashMap::from([
            (
                String::from("n"),
                WordVec::new(["an", "in", "no", "on", "one", "uno"]).unwrap()
            ),
            (String::from("nn"), WordVec::new(["inn", "anna"]).unwrap()),
            (String::from("hlf-tm"), WordVec::new(["half-time"]).unwrap())
        ]),
        dict.entries
    );

    dict.populate(stringify(["i", "one", "aio"])).unwrap();
    assert_eq!(
        HashMap::from([
            (
                String::from("n"),
                WordVec::new(["an", "in", "no", "on", "one", "uno"]).unwrap()
            ),
            (String::from("nn"), WordVec::new(["inn", "anna"]).unwrap()),
            (String::from("hlf-tm"), WordVec::new(["half-time"]).unwrap())
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
        let dict = Dict::new(case.input.clone()).unwrap();
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
        expect: Result<Option<&'static str>, WordResolveError>,
    }
    for case in vec![
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 0,
            expect: Ok(Some("in")),
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 1,
            expect: Ok(Some("on")),
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_position: 2,
            expect: Err(WordResolveError::borrowed(
                "no dictionary word at position 2 for fingerprint 'n'",
            )),
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "t",
            input_position: 2,
            expect: Ok(None),
        },
    ] {
        let dict = Dict::new(case.input_dict.clone()).unwrap();
        let actual = dict
            .resolve(case.input_fingerprint, case.input_position)
            .map(|option_of_string_ref| option_of_string_ref.map(String::as_str));
        assert_eq!(case.expect, actual, "for {case:?}");
    }
}

#[test]
fn position() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_fingerprint: &'static str,
        input_word: &'static str,
        expect: Option<u8>,
    }
    for case in vec![
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "in",
            expect: Some(0),
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "on",
            expect: Some(1),
        },
        Case {
            input_dict: vec!["in", "on"],
            input_fingerprint: "n",
            input_word: "an",
            expect: None,
        },
        Case {
            input_dict: vec![],
            input_fingerprint: "n",
            input_word: "an",
            expect: None,
        },
    ] {
        let dict = Dict::new(case.input_dict.clone()).unwrap();
        assert_eq!(
            case.expect,
            dict.position(case.input_fingerprint, case.input_word),
            "for {:?}",
            case
        )
    }
}

#[test]
fn contains_fingerprint() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_fingerprint: &'static str,
        expect: bool,
    }
    for case in vec![
        Case {
            input_dict: vec!["in"],
            input_fingerprint: "n",
            expect: true,
        },
        Case {
            input_dict: vec!["in"],
            input_fingerprint: "t",
            expect: false,
        },
    ] {
        let dict = Dict::new(case.input_dict.clone()).unwrap();
        assert_eq!(
            case.expect,
            dict.contains_fingerprint(case.input_fingerprint),
            "for {:?}",
            case
        );
    }
}

#[test]
fn populate_should_fill_to_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..255)
        .into_iter()
        .map(|i| format!("test-{}", "a".repeat(i)))
        .collect::<Vec<_>>();
    dict.populate(words).unwrap();
}

#[test]
#[should_panic(expected = "too many words")]
fn populate_should_not_fill_past_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..256)
        .into_iter()
        .map(|i| format!("test-{}", "a".repeat(i)))
        .collect::<Vec<_>>();
    dict.populate(words).unwrap();
}

#[test]
fn write_and_read_binary_image() {
    let dict = Dict::new(stringify(["in", "on", "at"])).unwrap();
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
    assert_eq!(
        Dict::new(stringify(["in", "on", "at", "the", "is", "of", "off"])).unwrap().entries,
        loaded.entries
    );
}

#[test]
fn read_from_text_file_error_implements_debug() {
    let error = ReadFromTextFileError::from(io::Error::new(ErrorKind::AddrInUse, "test"));
    assert!(format!("{error:?}").contains("Io"));

    let error = ReadFromTextFileError::from(OverflowError::borrowed("test"));
    assert!(format!("{error:?}").contains("DictOverflow"));
}

impl Dict {
    pub fn new(line: impl IntoIterator<Item = impl Into<String>>) -> Result<Dict, OverflowError> {
        let mut dict = Dict::default();
        dict.populate(line.into_iter().map(Into::into))?;
        Ok(dict)
    }
}

fn stringify<const N: usize>(strings: [&str; N]) -> Vec<String> {
    strings.iter().map(ToString::to_string).collect()
}

// $coverage:ignore-end
