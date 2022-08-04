use super::*;

#[test]
fn parse_line() {
    struct Case<'a> {
        line: &'a str,
        expect: Vec<(u8, &'a str)>,
    }
    let cases = vec![
        Case {
            line: "",
            expect: vec![],
        },
        Case {
            line: " ",
            expect: vec![],
        },
        Case {
            line: "the",
            expect: vec![(0, "the")],
        },
        Case {
            line: " the",
            expect: vec![(1, "the")],
        },
        Case {
            line: "  the",
            expect: vec![(2, "the")],
        },
        Case {
            line: "  the ",
            expect: vec![(2, "the")],
        },
        Case {
            line: "the quick brown fox",
            expect: vec![(0, "the"), (0, "quick"), (0, "brown"), (0, "fox")],
        },
        Case {
            line: " the   quick brown  fox ",
            expect: vec![(1, "the"), (2, "quick"), (0, "brown"), (1, "fox")],
        },
    ];

    for case in cases {
        let actual = EncodedWord::parse_line(case.line);
        let expect = case
            .expect
            .iter()
            .map(|&(leading_spaces, body)| EncodedWord::new(leading_spaces, body.to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(expect, actual, "for input '{}'", case.line);
    }
}

#[test]
fn reduce() {
    struct Case<'a> {
        word: &'a str,
        expect: Reduction,
    }
    let cases = vec![
        Case {
            word: "fox",
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            word: " foxy ",
            expect: Reduction { fingerprint: String::from(" fxy "), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            word: "Fox",
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: true, trailing_capitals: 0},
        },
        Case {
            word: "FoX",
            expect: Reduction { fingerprint: String::from("fx"), leading_capital: true, trailing_capitals: 1},
        },
        Case {
            word: " FoX",
            expect: Reduction { fingerprint: String::from(" fx"), leading_capital: false, trailing_capitals: 2},
        },
    ];

    for case in cases {
        let actual = Reduction::from(case.word);
        assert_eq!(case.expect, actual, "for input '{}'", case.word);
    }
}

#[test]
fn split_word() {
    struct Case<'a> {
        word: &'a str,
        expect: (&'a str, &'a str),
    }
    let cases = vec![
        Case {
            word: "foo",
            expect: ("foo", "")
        },
        Case {
            word: "foo!🦄",
            expect: ("foo", "!🦄")
        },
        Case {
            word: "¿foo?",
            expect: ("", "¿foo?")
        },
        Case {
            word: "123",
            expect: ("", "123")
        },
        Case {
            word: "foo1.1",
            expect: ("foo", "1.1")
        }
    ];

    for case in cases {
        let expected = SplitWord { prefix: case.expect.0, suffix: case.expect.1 };
        let actual = SplitWord::from(case.word);
        assert_eq!(expected, actual, "for input '{}'", case.word);
    }
}
// struct HashMapBuilder<K, V>(HashMap<K, V>);
//
// impl <K, V> HashMapBuilder<K, V> {
//     fn new() -> Self { Self(HashMap::new()) }
//
//
// }
