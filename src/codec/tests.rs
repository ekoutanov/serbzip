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
        let actual = Word::parse_line(case.line);
        let expect = case
            .expect
            .iter()
            .map(|&(lead, body)| Word(lead, body.to_owned()))
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
            expect: Reduction { reduced: String::from("fx"), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            word: " foxy ",
            expect: Reduction { reduced: String::from(" fxy "), leading_capital: false, trailing_capitals: 0},
        },
        Case {
            word: "Fox",
            expect: Reduction { reduced: String::from("Fx"), leading_capital: true, trailing_capitals: 0},
        },
        Case {
            word: "FoX",
            expect: Reduction { reduced: String::from("FX"), leading_capital: true, trailing_capitals: 1},
        },
        Case {
            word: " FoX",
            expect: Reduction { reduced: String::from(" FX"), leading_capital: false, trailing_capitals: 2},
        },
    ];

    for case in cases {
        let actual = Reduction::from(case.word);
        assert_eq!(case.expect, actual, "for input '{}'", case.word);
    }
}

#[test]
fn dict_populate_incremental() {
    let mut dict = Dict::default();

    dict.populate(&stringify(["uno", "one", "no"]));
    assert_eq!(
        HashMap::from([(String::from("n"), stringify(["no", "uno", "one"]))]),
        dict.entries
    );

    dict.populate(&stringify(["Anna"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "uno", "one"])),
            (String::from("nn"), stringify(["anna"]))
        ]),
        dict.entries
    );

    dict.populate(&stringify(["half-time"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "uno", "one"])),
            (String::from("nn"), stringify(["anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );

    dict.populate(&stringify(["on", "an", "in", "inn"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "on", "an", "in", "uno", "one"])),
            (String::from("nn"), stringify(["inn", "anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );
}

fn stringify<const N: usize>(strings: [&str; N]) -> Vec<String> {
    strings.iter().map(|&s| String::from(s)).collect()
}

// struct HashMapBuilder<K, V>(HashMap<K, V>);
//
// impl <K, V> HashMapBuilder<K, V> {
//     fn new() -> Self { Self(HashMap::new()) }
//
//
// }
