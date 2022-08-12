use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use serbzip::codecs::balkanoid::Dict;
use crate::common::generate_random_path;

mod common;

#[test]
fn read_write_read_small_dicts() {
    test_round_trip("test_data/dict_eng_small.txt");
    test_round_trip("test_data/dict_rus_small.txt");
}

#[ignore]
#[test]
fn read_write_read_large_dicts() {
    test_round_trip("test_data/dict_eng_standard.txt");
    test_round_trip("test_data/dict_eng_large.txt");
    test_round_trip("test_data/dict_rus_standard.txt");
}

fn test_round_trip(txt_file: &str) {
    let txt_path = Path::new(txt_file);
    let dict = Dict::read_from_text_file(&mut BufReader::new(File::open(txt_path).unwrap())).unwrap();

    let bin_path = generate_random_path("blk");
    dict.write_to_binary_image(&mut BufWriter::new(File::create(&bin_path).unwrap())).unwrap();

    let loaded = Dict::read_from_binary_image(&mut BufReader::new(File::open(&bin_path).unwrap())).unwrap();
    assert_eq!(dict, loaded);
}