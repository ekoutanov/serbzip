use std::collections::HashMap;
use super::*;

#[test]
fn dict_populate_incremental() {
    let mut dict = Dict::default();

    dict.populate(stringify(["uno", "one", "no"]));
    assert_eq!(
        HashMap::from([(String::from("n"), stringify(["no", "uno", "one"]))]),
        dict.entries
    );

    dict.populate(stringify(["Anna"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "uno", "one"])),
        ]),
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
            (String::from("n"), stringify(["no", "on", "an", "in", "uno", "one"])),
            (String::from("nn"), stringify(["inn", "anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );

    dict.populate(stringify(["i", "aio"]));
    assert_eq!(
        HashMap::from([
            (String::from("n"), stringify(["no", "on", "an", "in", "uno", "one"])),
            (String::from("nn"), stringify(["inn", "anna"])),
            (String::from("hlf-tm"), stringify(["half-time"]))
        ]),
        dict.entries
    );
}

#[test]
fn populate_should_fill_to_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..255).into_iter().map(|i| format!("test-{}", "a".repeat(i))).collect::<Vec<_>>();
    dict.populate(words);
}

#[test]
#[should_panic(expected="too many words associated")]
fn populate_should_not_fill_past_fingerprint_limit() {
    let mut dict = Dict::default();
    let words = (0..256).into_iter().map(|i| format!("test-{}", "a".repeat(i))).collect::<Vec<_>>();
    dict.populate(words);
}

fn stringify<const N: usize>(strings: [&str; N]) -> Vec<String> {
    strings.iter().map(|&s| String::from(s)).collect()
}