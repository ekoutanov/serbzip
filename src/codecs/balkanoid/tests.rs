use super::*;

// $coverage:ignore-start

#[test]
fn parse_line() {
    struct Case {
        input: &'static str,
        expect: Vec<(u8, &'static str)>,
    }
    for case in vec![
        Case {
            input: "",
            expect: vec![],
        },
        Case {
            input: " ",
            expect: vec![],
        },
        Case {
            input: "the",
            expect: vec![(0, "the")],
        },
        Case {
            input: " the",
            expect: vec![(1, "the")],
        },
        Case {
            input: "  the",
            expect: vec![(2, "the")],
        },
        Case {
            input: "  the ",
            expect: vec![(2, "the")],
        },
        Case {
            input: "the quick brown fox",
            expect: vec![(0, "the"), (0, "quick"), (0, "brown"), (0, "fox")],
        },
        Case {
            input: " the   quick brown  fox ",
            expect: vec![(1, "the"), (2, "quick"), (0, "brown"), (1, "fox")],
        },
    ] {
        let actual = EncodedWord::parse_line(case.input);
        let expect = case
            .expect
            .iter()
            .map(|&(leading_spaces, body)| EncodedWord::new(leading_spaces, body.to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(expect, actual, "for input '{}'", case.input);
    }
}

#[test]
fn reduce() {
    struct Case {
        input: &'static str,
        expect: Reduction,
    }
    for case in vec![
        Case {
            input: "fox",
            expect: Reduction {
                fingerprint: String::from("fx"),
                leading_capital: false,
                trailing_capitals: 0,
            },
        },
        Case {
            input: " foxy ",
            expect: Reduction {
                fingerprint: String::from(" fxy "),
                leading_capital: false,
                trailing_capitals: 0,
            },
        },
        Case {
            input: "Fox",
            expect: Reduction {
                fingerprint: String::from("fx"),
                leading_capital: true,
                trailing_capitals: 0,
            },
        },
        Case {
            input: "FoX",
            expect: Reduction {
                fingerprint: String::from("fx"),
                leading_capital: true,
                trailing_capitals: 1,
            },
        },
        Case {
            input: " FoX",
            expect: Reduction {
                fingerprint: String::from(" fx"),
                leading_capital: false,
                trailing_capitals: 2,
            },
        },
        Case {
            input: "",
            expect: Reduction {
                fingerprint: String::from(""),
                leading_capital: false,
                trailing_capitals: 0,
            },
        },
        Case {
            input: " ",
            expect: Reduction {
                fingerprint: String::from(" "),
                leading_capital: false,
                trailing_capitals: 0,
            },
        },
    ] {
        let actual = Reduction::from(case.input);
        assert_eq!(case.expect, actual, "for input '{}'", case.input);
    }
}

#[test]
fn reduction_take_if_lowercase() {
    assert_eq!(
        Reduction::from("test").take_if_lowercase(),
        Some(Reduction {
            fingerprint: String::from("tst"),
            leading_capital: false,
            trailing_capitals: 0
        })
    );

    assert_eq!(Reduction::from("tesT").take_if_lowercase(), None);
}

#[test]
fn punctuate_word() {
    struct Case {
        input: &'static str,
        expect: (&'static str, &'static str),
    }
    for case in vec![
        Case {
            input: "foo",
            expect: ("foo", ""),
        },
        Case {
            input: "foo!🦄",
            expect: ("foo", "!🦄"),
        },
        Case {
            input: "¿foo?",
            expect: ("", "¿foo?"),
        },
        Case {
            input: "123",
            expect: ("", "123"),
        },
        Case {
            input: "foo1.1",
            expect: ("foo", "1.1"),
        },
        Case {
            input: "\\x!",
            expect: ("\\x", "!"),
        },
        Case {
            input: "\\!",
            expect: ("\\", "!"),
        },
        Case {
            input: "\\",
            expect: ("\\", ""),
        },
        Case {
            input: "\\\\",
            expect: ("\\", "\\"),
        },
        Case {
            input: "\\def\\",
            expect: ("\\def", "\\"),
        },
        Case {
            input: "яблоко",
            expect: ("яблоко", ""),
        },
        Case {
            input: "яблоко!",
            expect: ("яблоко", "!"),
        },
        Case {
            input: "\\яблоко!",
            expect: ("\\яблоко", "!"),
        },
    ] {
        let expected = PunctuatedWord {
            prefix: Cow::Borrowed(case.expect.0),
            suffix: Cow::Borrowed(case.expect.1),
        };
        let actual = PunctuatedWord::from(case.input);
        assert_eq!(expected, actual, "for input '{}'", case.input);
    }
}

#[test]
fn compress_expand_word() {
    struct Case {
        input_dict: Vec<&'static str>,
        input_word: &'static str,
        expect_encoded: (u8, &'static str),
        expect_proper_capitalisation: bool,
    }
    for case in vec![
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "count",
            expect_encoded: (1, "cnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Count",
            expect_encoded: (1, "Cnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CoUnt",
            expect_encoded: (1, "CNT"),
            expect_proper_capitalisation: false
        },
        Case {
            input_dict: vec!["canet"],
            input_word: "CoUnt",
            expect_encoded: (0, "CoUnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CounT",
            expect_encoded: (1, "CNT"),
            expect_proper_capitalisation: false
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "canet",
            expect_encoded: (0, "cnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "cont",
            expect_encoded: (0, "cont"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Cont",
            expect_encoded: (0, "Cont"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "ConT",
            expect_encoded: (0, "ConT"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "cnt",
            expect_encoded: (0, "\\cnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Cnt",
            expect_encoded: (0, "\\Cnt"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CnT",
            expect_encoded: (0, "\\CnT"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "mark",
            expect_encoded: (0, "mark"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![],
            input_word: "kgb",
            expect_encoded: (0, "kgb"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["kagoob"],
            input_word: "kgb",
            expect_encoded: (0, "\\kgb"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["kgb"],
            input_word: "kgb",
            expect_encoded: (0, "kgb"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "a",
            expect_encoded: (0, "a"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "aio",
            expect_encoded: (0, "aio"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "AIO",
            expect_encoded: (0, "AIO"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "A",
            expect_encoded: (0, "A"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "s.",
            expect_encoded: (0, "\\s."),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "S.",
            expect_encoded: (0, "\\S."),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["prpr"],
            input_word: "-prPr-",
            expect_encoded: (0, "-prPr-"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["ra"],
            input_word: "ra",
            expect_encoded: (0, "r"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["ra"],
            input_word: "Ra",
            expect_encoded: (0, "R"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["ra"],
            input_word: "RA",
            expect_encoded: (0, "R"),
            expect_proper_capitalisation: false
        },
        Case {
            input_dict: vec![""],
            input_word: "\\",
            expect_encoded: (0, "\\\\"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![""],
            input_word: "\\\\",
            expect_encoded: (0, "\\\\\\"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![""],
            input_word: "attaché",
            expect_encoded: (0, "attaché"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![""],
            input_word: "attaché,",
            expect_encoded: (0, "attaché,"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["яблоко"],
            input_word: "яблоко",
            expect_encoded: (0, "блк"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["яблоко"],
            input_word: "Яблоко",
            expect_encoded: (0, "Блк"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec!["яблоко", "яблоки"],
            input_word: "Яблоки",
            expect_encoded: (0, "Блк"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![""],
            input_word: "уж",
            expect_encoded: (0, "уж"),
            expect_proper_capitalisation: true
        },
        Case {
            input_dict: vec![],
            input_word: "\\def\\ebook{33283}",
            expect_encoded: (0, "\\\\def\\ebook{33283}"),
            expect_proper_capitalisation: true
        },
    ] {
        let dict = Dict::new(case.input_dict.clone());
        let expected = EncodedWord::new(case.expect_encoded.0, case.expect_encoded.1.to_owned());
        let actual = compress_word(&dict, case.input_word);
        assert_eq!(
            expected, actual,
            "[compression] for input '{}' with dict {:?}",
            case.input_word, &case.input_dict
        );

        let expanded = expand_word(&dict, actual).unwrap();
        assert_eq!(
            case.input_word.to_lowercase(),
            expanded.to_lowercase(),
            "[expansion, case-insensitive check] for input '{}' with dict {:?}",
            case.input_word,
            &case.input_dict
        );
        if case.expect_proper_capitalisation {
            assert_eq!(
                case.input_word,
                expanded,
                "[expansion, case-sensitive check, expect equal] for input '{}' with dict {:?}",
                case.input_word,
                &case.input_dict
            );
        } else {
            assert_ne!(
                case.input_word,
                expanded,
                "[expansion, case-sensitive check, expect unequal] for input '{}' with dict {:?}",
                case.input_word,
                &case.input_dict
            );
        }
    }
}

#[test]
fn expand_word_cannot_resolve() {
    let dict = Dict::new(vec!["in", "on", "as", "is"]);
    let result = expand_word(
        &dict,
        EncodedWord {
            leading_spaces: 2,
            body: String::from("n"),
        },
    );
    assert_eq!(
        Err(WordResolveError::borrowed(
            "no dictionary word at position 2 for fingerprint 'n'"
        )),
        result
    );
}

#[test]
fn compress_expand_line() {
    #[derive(Debug)]
    struct Case {
        input_dict: Vec<&'static str>,
        input_line: &'static str,
        expect: &'static str,
    }
    for case in vec![
        Case {
            input_dict: vec!["in", "on", "as", "is"],
            input_line: "he came in, as one",
            expect: "he came n, s one",
        },
        Case {
            input_dict: vec!["in", "on", "one", "way"],
            input_line: "he came in, as one, and went on his way!",
            expect: "he came n, as   n, and went  n his wy!",
        },
        Case {
            input_dict: vec!["in", "on", "as", "is"],
            input_line: "He came In, As One",
            expect: "He came N, S One",
        },
    ] {
        let dict = Dict::new(case.input_dict.clone());
        let codec = Balkanoid::new(&dict);
        let actual = codec.compress_line(case.input_line);
        assert_eq!(case.expect, actual, "[compression] for {case:?}");

        let expanded = codec.expand_line(&actual).unwrap();
        assert_eq!(
            case.input_line.to_lowercase(),
            expanded.to_lowercase(),
            "[expansion] for {case:?}"
        )
    }
}

#[test]
fn expand_line_cannot_resolve() {
    let dict = Dict::new(vec!["in", "on", "as", "is"]);
    let result = Balkanoid::new(&dict).expand_line("  n");
    assert_eq!(
        Err(WordResolveError::borrowed(
            "no dictionary word at position 2 for fingerprint 'n'"
        )),
        result
    );
}

#[test]
fn compaction_rule_implements_debug() {
    let formatted = format!("{:?}", CompactionRule::Conflict);
    assert_eq!("Conflict", formatted);
}

// $coverage:ignore-end