use super::*;

#[test]
fn parse_line() {
    struct Case<'a> {
        input: &'a str,
        expect: Vec<(u8, &'a str)>,
    }
    let cases = vec![
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
    ];

    for case in cases {
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
    struct Case<'a> {
        input: &'a str,
        expect: Reduction,
    }
    let cases = vec![
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
    ];

    for case in cases {
        let actual = Reduction::from(case.input);
        assert_eq!(case.expect, actual, "for input '{}'", case.input);
    }
}

#[test]
fn split_word() {
    struct Case<'a> {
        input: &'a str,
        expect: (&'a str, &'a str),
    }
    let cases = vec![
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
    ];

    for case in cases {
        let expected = SplitWord { prefix: Cow::Borrowed(case.expect.0), suffix: Cow::Borrowed(case.expect.1) };
        let actual = SplitWord::from(case.input);
        assert_eq!(expected, actual, "for input '{}'", case.input);
    }
}

#[test]
fn compress_expand_word() {
    struct Case<'a> {
        dict_words: Vec<&'a str>,
        input: &'a str,
        expect: (u8, &'a str),
    }
    let cases = vec![
        Case {
            dict_words: vec!["count", "canet"],
            input: "count",
            expect: (0, "cnt")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "Count",
            expect: (0, "Cnt")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "CoUnt",
            expect: (0, "CNT")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "CounT",
            expect: (0, "CNT")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "canet",
            expect: (1, "cnt")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "cont",
            expect: (0, "cont")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "Cont",
            expect: (0, "Cont")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "ConT",
            expect: (0, "CONT")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "cnt",
            expect: (0, "\\cnt")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "Cnt",
            expect: (0, "\\Cnt")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "CnT",
            expect: (0, "\\CnT")
        },
        Case {
            dict_words: vec!["count", "canet"],
            input: "mark",
            expect: (0, "mark")
        },
        Case {
            dict_words: vec![],
            input: "kgb",
            expect: (0, "kgb")
        },
        Case {
            dict_words: vec!["kagoob"],
            input: "kgb",
            expect: (0, "\\kgb")
        },
        Case {
            dict_words: vec!["kgb"],
            input: "kgb",
            expect: (0, "kgb")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "a",
            expect: (0, "a")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "aio",
            expect: (0, "aio")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "AIO",
            expect: (0, "AIO")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "A",
            expect: (0, "A")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "s.",
            expect: (0, "\\s.")
        },
        Case {
            dict_words: vec!["as", "is"],
            input: "S.",
            expect: (0, "\\S.")
        },
        Case {
            dict_words: vec!["prpr"],
            input: "-prPr-",
            expect: (0, "-prPr-")
        },
        Case {
            dict_words: vec![""],
            input: "\\",
            expect: (0, "\\")
        },
        Case {
            dict_words: vec![""],
            input: "\\\\",
            expect: (0, "\\\\")
        },
        Case {
            dict_words: vec![""],
            input: "attaché",
            expect: (0, "attaché")
        },
        Case {
            dict_words: vec![""],
            input: "attaché,",
            expect: (0, "attaché,")
        },
        Case {
            dict_words: vec!["яблоко"],
            input: "яблоко",
            expect: (0, "блк")
        },
        Case {
            dict_words: vec!["яблоко"],
            input: "Яблоко",
            expect: (0, "Блк")
        },
        Case {
            dict_words: vec!["яблоко", "яблоки"],
            input: "Яблоки",
            expect: (1, "Блк")
        },
        Case {
            dict_words: vec![""],
            input: "уж",
            expect: (0, "уж")
        }
    ];

    for Case { dict_words, input , expect} in cases {
        let mut dict = Dict::default();
        let dict_words = dict_words.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>();
        dict.populate(dict_words);
        let expected = EncodedWord::new(expect.0, expect.1.to_owned());
        let actual = compress_word(&dict, input);
        assert_eq!(expected, actual, "[compression] for input' '{input}' with {dict:?}");

        let expanded = expand_word(&dict, actual).unwrap();
        assert_eq!(input.to_lowercase(), expanded.to_lowercase(), "[expansion] for input '{input}' with {dict:?}");
    }
}
// struct HashMapBuilder<K, V>(HashMap<K, V>);
//
// impl <K, V> HashMapBuilder<K, V> {
//     fn new() -> Self { Self(HashMap::new()) }
//
//
// }
