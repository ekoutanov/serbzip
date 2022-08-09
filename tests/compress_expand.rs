use rand::RngCore;
use serbzip::codecs::balkanoid::{Balkanoid, Dict};
use serbzip::codecs::Codec;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[test]
fn compress_and_expand_small_docs() {
    let dict = read_default_dict();
    test_compress_and_expand(&dict, "test_data/antigonish.txt");
    test_compress_and_expand(&dict, "test_data/the_raven.txt");
    test_compress_and_expand(&dict, "test_data/sherlock_holmes.txt");
}

#[test]
#[ignore]
fn compress_and_expand_large_docs() {
    let dict = read_default_dict();
    test_compress_and_expand(&dict, "test_data/mormon.txt");
    test_compress_and_expand(&dict, "test_data/anna_karenina_eng.txt");
    test_compress_and_expand(&dict, "test_data/anna_karenina_rus.txt");
}

fn read_default_dict() -> Dict {
    Dict::read_from_binary_image(&mut BufReader::new(File::open("dict.img").unwrap())).unwrap()
}

fn test_compress_and_expand(dict: &Dict, original_file: &str) {
    let original_path = Path::new(original_file);
    let compressed_path = generate_random_path("sz");
    let expanded_path = generate_random_path("txt");
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
            //println!("source: '{source_line}', target: '{target_line}'");
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
                    println!("[{original_file}] capitalisation mismatch at line {line_no}, word {word_no}: '{src_word}' ≠ '{tgt_word}");
                }
                word_no += 1;
            }

            line_no += 1;
        }
    }

    fs::remove_file(compressed_path).unwrap();
}

fn generate_random_path(extension: &str) -> PathBuf {
    let path_buf = std::env::temp_dir();
    let random = rand::thread_rng().next_u64();
    path_buf.join(format!("test-{random:X}.{extension}"))
}
