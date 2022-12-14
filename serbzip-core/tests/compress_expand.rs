mod common;

use serbzip_core::codecs::balkanoid::{Balkanoid, Dict};
use serbzip_core::codecs::Codec;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path};

#[test]
fn compress_and_expand_small_docs() {
    let dict = read_default_dict();
    test_compress_and_expand(&dict, "../test_data/antigonish.txt");
    test_compress_and_expand(&dict, "../test_data/no_man_is_an_island.txt");
    test_compress_and_expand(&dict, "../test_data/a_dream_within_a_dream.txt");
    test_compress_and_expand(&dict, "../test_data/u_lukomorya.txt");
    test_compress_and_expand(&dict, "../test_data/lyublyu_tebya.txt");
    test_compress_and_expand(&dict, "../test_data/subterranean_vaults.txt");
    test_compress_and_expand(&dict, "../test_data/the_raven.txt");
    test_compress_and_expand(&dict, "../README.md");
}

#[test]
#[ignore]
fn compress_and_expand_medium_docs() {
    let dict = read_default_dict();
    test_compress_and_expand(&dict, "../test_data/alice_in_wonderland.txt");
    test_compress_and_expand(&dict, "../test_data/calculus_made_easy.txt");
    test_compress_and_expand(&dict, "../test_data/dracula.txt");
    test_compress_and_expand(&dict, "../test_data/effective_kafka.txt");
    test_compress_and_expand(&dict, "../test_data/frankenstein.txt");
    test_compress_and_expand(&dict, "../test_data/metamorphosis.txt");
    test_compress_and_expand(&dict, "../test_data/odnazhdy.txt");
    test_compress_and_expand(&dict, "../test_data/pride_and_prejudice.txt");
    test_compress_and_expand(&dict, "../test_data/sherlock_holmes.txt");
    test_compress_and_expand(&dict, "../test_data/the_prince.txt");
}

#[test]
#[ignore]
fn compress_and_expand_large_docs() {
    let dict = read_default_dict();
    test_compress_and_expand(&dict, "../test_data/anna_karenina_eng.txt");
    test_compress_and_expand(&dict, "../test_data/anna_karenina_rus.txt");
    test_compress_and_expand(&dict, "../test_data/count_of_monte_cristo.txt");
    test_compress_and_expand(&dict, "../test_data/crime_and_punishment_eng.txt");
    test_compress_and_expand(&dict, "../test_data/moby_dick.txt");
    test_compress_and_expand(&dict, "../test_data/mormon.txt");
    test_compress_and_expand(&dict, "../test_data/new_testament.txt");
    test_compress_and_expand(&dict, "../test_data/jane_eyre.txt");
    test_compress_and_expand(&dict, "../test_data/war_and_peace_eng.txt");
    test_compress_and_expand(&dict, "../test_data/war_and_peace_rus.txt");
}

fn read_default_dict() -> Dict {
    Dict::read_from_binary_image(&mut BufReader::new(File::open("../dict.blk").unwrap())).unwrap()
}

fn test_compress_and_expand(dict: &Dict, original_file: &str) {
    let original_path = Path::new(original_file);
    let compressed_path = common::TempPath::with_extension("sz");
    let expanded_path = common::TempPath::with_extension("txt");
    let codec = Balkanoid::new(dict);

    {
        // compress a given source file to a temporary .sz file
        let mut r = BufReader::new(File::open(&original_path).unwrap());
        let mut w = BufWriter::new(File::create(&compressed_path).unwrap());
        println!("compressing {original_path:?} to {compressed_path:?}");
        codec.compress(&mut r, &mut w).unwrap();
    }

    {
        // expand the previously compressed file
        let mut r = BufReader::new(File::open(&compressed_path).unwrap());
        let mut w = BufWriter::new(File::create(&expanded_path).unwrap());
        println!("expanding {compressed_path:?} to {expanded_path:?}");
        codec.expand(&mut r, &mut w).unwrap();
    }

    {
        // verify that the source file and the expanded file are equivalent
        let mut src = BufReader::new(File::open(&original_path).unwrap());
        let mut tgt = BufReader::new(File::open(&expanded_path).unwrap());
        let mut line_no = 1u32;
        loop {
            let (mut src_buf, mut tgt_buf) = (String::new(), String::new());
            let src_bytes = src.read_line(&mut src_buf).unwrap();
            let tgt_bytes = tgt.read_line(&mut tgt_buf).unwrap();

            if src_bytes == 0 && tgt_bytes == 0 {
                break;
            }

            let (src_line, tgt_line) = (&src_buf[0..src_bytes - 1], &tgt_buf[0..tgt_bytes - 1]);
            let mut src_words = src_line.split_whitespace();
            let mut tgt_words = tgt_line.split_whitespace();

            let mut word_no = 1u32;
            loop {
                let (src_word, tgt_word) = (src_words.next(), tgt_words.next());
                if src_word == None && tgt_word == None {
                    break;
                }
                assert!(
                    !matches!(src_word, None),
                    "[{original_file}] missing source word {word_no} at line {line_no}"
                );
                assert!(
                    !matches!(tgt_word, None),
                    "[{original_file}] missing target word {word_no} at line {line_no}"
                );
                let (src_word, tgt_word) = (src_word.unwrap(), tgt_word.unwrap());
                assert_eq!(
                    src_word.to_lowercase(),
                    tgt_word.to_lowercase(),
                    "[{original_file}] word mismatch at line {line_no}, word {word_no}"
                );
                if src_word != tgt_word {
                    // The algorithm doesn't guarantee that the capitalisation will match -- print differences for debugging.
                    // Note: this is not a failure.
                    println!("[{original_file}] capitalisation mismatch at line {line_no}, word {word_no}: '{src_word}' ??? '{tgt_word}");
                }
                word_no += 1;
            }

            line_no += 1;
        }
    }
}
