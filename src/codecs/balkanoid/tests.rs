use super::*;

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
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            input: " foxy ",
            expect: Reduction { fingerprint: String::from(" fxy "), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            input: "Fox",
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: true, trailing_capitals: 0},
        },
        Case {
            input: "FoX",
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: true, trailing_capitals: 1},
        },
        Case {
            input: " FoX",
            expect: Reduction { fingerprint: String::from(" fx"), leading_capital: false, trailing_capitals: 2},
        },
    ] {
        let actual = Reduction::from(case.input);
        assert_eq!(case.expect, actual, "for input '{}'", case.input);
    }
}

#[test]
fn split_word() {
    struct Case {
        input: &'static str,
        expect: (&'static str, &'static str),
    }
    for case in vec![
        Case {
            input: "foo",
            expect: ("foo", "")
        },
        Case {
            input: "foo!🦄",
            expect: ("foo", "!🦄")
        },
        Case {
            input: "¿foo?",
            expect: ("", "¿foo?")
        },
        Case {
            input: "123",
            expect: ("", "123")
        },
        Case {
            input: "foo1.1",
            expect: ("foo", "1.1")
        },
        Case {
            input: "\\x!",
            expect: ("\\x", "!")
        },
        Case {
            input: "\\!",
            expect: ("\\", "!")
        },
        Case {
            input: "\\",
            expect: ("\\", "")
        },
        Case {
            input: "яблоко",
            expect: ("яблоко", "")
        },
        Case {
            input: "яблоко!",
            expect: ("яблоко", "!")
        },
        Case {
            input: "\\яблоко!",
            expect: ("\\яблоко", "!")
        },
    ] {
        let expected = SplitWord { prefix: Cow::Borrowed(case.expect.0), suffix: Cow::Borrowed(case.expect.1) };
        let actual = SplitWord::from(case.input);
        assert_eq!(expected, actual, "for input '{}'", case.input);
    }
}

#[test]
fn compress_expand_word() {
    struct Case {
        input_dict: Vec<&'static str>,
        input_word: &'static str,
        expect: (u8, &'static str),
    }
    for case in vec![
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "count",
            expect: (0, "cnt")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Count",
            expect: (0, "Cnt")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CoUnt",
            expect: (0, "CNT")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CounT",
            expect: (0, "CNT")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "canet",
            expect: (1, "cnt")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "cont",
            expect: (0, "cont")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Cont",
            expect: (0, "Cont")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "ConT",
            expect: (0, "CONT")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "cnt",
            expect: (0, "\\cnt")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "Cnt",
            expect: (0, "\\Cnt")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "CnT",
            expect: (0, "\\CnT")
        },
        Case {
            input_dict: vec!["count", "canet"],
            input_word: "mark",
            expect: (0, "mark")
        },
        Case {
            input_dict: vec![],
            input_word: "kgb",
            expect: (0, "kgb")
        },
        Case {
            input_dict: vec!["kagoob"],
            input_word: "kgb",
            expect: (0, "\\kgb")
        },
        Case {
            input_dict: vec!["kgb"],
            input_word: "kgb",
            expect: (0, "kgb")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "a",
            expect: (0, "a")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "aio",
            expect: (0, "aio")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "AIO",
            expect: (0, "AIO")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "A",
            expect: (0, "A")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "s.",
            expect: (0, "\\s.")
        },
        Case {
            input_dict: vec!["as", "is"],
            input_word: "S.",
            expect: (0, "\\S.")
        },
        Case {
            input_dict: vec!["prpr"],
            input_word: "-prPr-",
            expect: (0, "-prPr-")
        },
        Case {
            input_dict: vec![""],
            input_word: "\\",
            expect: (0, "\\")
        },
        Case {
            input_dict: vec![""],
            input_word: "\\\\",
            expect: (0, "\\\\")
        },
        Case {
            input_dict: vec![""],
            input_word: "attaché",
            expect: (0, "attaché")
        },
        Case {
            input_dict: vec![""],
            input_word: "attaché,",
            expect: (0, "attaché,")
        },
        Case {
            input_dict: vec!["яблоко"],
            input_word: "яблоко",
            expect: (0, "блк")
        },
        Case {
            input_dict: vec!["яблоко"],
            input_word: "Яблоко",
            expect: (0, "Блк")
        },
        Case {
            input_dict: vec!["яблоко", "яблоки"],
            input_word: "Яблоки",
            expect: (1, "Блк")
        },
        Case {
            input_dict: vec![""],
            input_word: "уж",
            expect: (0, "уж")
        }
    ] {
        let dict = Dict::from(case.input_dict.clone());
        let expected = EncodedWord::new(case.expect.0, case.expect.1.to_owned());
        let actual = compress_word(&dict, case.input_word);
        assert_eq!(expected, actual, "[compression] for input '{}' with dict {:?}", case.input_word, &case.input_dict);

        let expanded = expand_word(&dict, actual).unwrap();
        assert_eq!(case.input_word.to_lowercase(), expanded.to_lowercase(), "[expansion] for input '{}' with dict {:?}", case.input_word, &case.input_dict);
    }
}
