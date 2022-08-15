`serbzip-core`
===
The library package for [serb.zip](https://github.com/ekoutanov/serbzip).

[![Crates.io](https://img.shields.io/crates/v/serbzip_core?style=flat-square&logo=rust)](https://crates.io/crates/serbzip-core)
[![docs.rs](https://img.shields.io/badge/docs.rs-serbzip_core-blue?style=flat-square&logo=docs.rs)](https://docs.rs/serbzip-core)
[![Build Status](https://img.shields.io/github/workflow/status/ekoutanov/serbzip/Cargo%20build?style=flat-square&logo=github)](https://github.com/ekoutanov/serbzip/actions/workflows/master.yml)
[![codecov](https://img.shields.io/codecov/c/github/ekoutanov/serbzip/master?style=flat-square&logo=codecov)](https://codecov.io/gh/ekoutanov/serbzip)

# Getting started
## Add dependency
```sh
cargo add serbzip-core
```

## Compress and decompress some text
We'll use the Balkanoid codec for this example.

The sample code assumes we have `../dict.blk` and `../test_data/antigonish.txt` to play with.

```rust
use std::fs::File;
use std::io;
use std::io::BufReader;
use serbzip_core::codecs::balkanoid::{Balkanoid, Dict};
use serbzip_core::codecs::Codec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // this codec needs a dictionary to work from
    let mut dict_reader = BufReader::new(File::open("../dict.blk")?);
    let dict = Dict::read_from_binary_image(&mut dict_reader)?;
    let codec = Balkanoid::new(&dict);

    // compress a line and check the output
    let input_line = "Ah, distinctly I remember it was in the bleak December";
    let compressed_line = codec.compress_line(input_line);
    assert_eq!(compressed_line, "H, dstnctly I rmmbr  t ws  n th   blk Dcmbr");

    // expand the line; check that it matches the original
    let expanded_line = codec.expand_line(&compressed_line)?;
    assert_eq!(input_line, expanded_line);

    // codecs also have helper methods for parsing I/O streams
    let mut input_reader = BufReader::new(File::open("../test_data/antigonish.txt")?);
    let mut output_writer = io::Cursor::new(Vec::new());
    codec.compress(&mut input_reader, &mut output_writer)?;
    let compressed_document = String::from_utf8(output_writer.into_inner())?;
    assert_ne!("", compressed_document);
    
    Ok(())
}
```