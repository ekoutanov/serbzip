use std::io::{Cursor};
use crate::codecs::armenoid::Armenoid;
use crate::codecs::Codec;

#[test]
fn test_compress_without_error() {
    let mut r = Cursor::new("Artsakh is ours".as_bytes());
    let mut w = Cursor::new(Vec::<u8>::new());
    let codec = Armenoid::default();
    codec.compress(&mut r, &mut w).unwrap();
    assert_eq!(String::from("inch inch inch\n"), String::from_utf8(w.into_inner()).unwrap());
}

#[test]
fn test_expand_without_error() {
    let mut r = Cursor::new("inch inch inch".as_bytes());
    let mut w = Cursor::new(Vec::<u8>::new());
    let codec = Armenoid::default();
    codec.expand(&mut r, &mut w).unwrap();
    assert_eq!(String::from("what what what\n"), String::from_utf8(w.into_inner()).unwrap());
}