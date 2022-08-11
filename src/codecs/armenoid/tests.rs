use crate::codecs::armenoid::Armenoid;
use crate::codecs::Codec;

// $coverage:ignore-start

#[test]
fn compress_line() {
    assert_eq!(
        String::from("inch inch inch"),
        Armenoid::default().compress_line("apple bird bee")
    );
}

#[test]
fn expand_line() {
    assert_eq!(
        Ok(String::from("what what what")),
        Armenoid::default().expand_line("inch inch inch")
    );
}

// $coverage:ignore-end
