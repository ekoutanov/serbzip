//! A lightweight framework for transcoding text from one lexical form to another.
//!
//! The apparatus that performs this trancoding is called a **codec**. It is described by the
//! [`codecs::Codec`] trait. A prominent example of such a codec is **Balkanoid** —
//! a quasi-lossless Balkanoidal meta-lingual compressor.
//!
//! This crate provides mechanisms for creating codecs, as well as one or more useful
//! codec implementations.
//!
//! # Examples
//! Compression and expansion of text using the [`codecs::balkanoid::Balkanoid`]
//! codec:
//! ```
//! use std::fs::File;
//! use std::io;
//! use std::io::BufReader;
//! use serbzip_core::codecs::balkanoid::{Balkanoid, Dict};
//! use serbzip_core::codecs::Codec;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // this codec needs a dictionary to work from
//! let mut dict_reader = BufReader::new(File::open("../dict.blk")?);
//! let dict = Dict::read_from_binary_image(&mut dict_reader)?;
//! let codec = Balkanoid::new(&dict);
//!
//! // compress a line and check the output
//! let input_line = "Ah, distinctly I remember it was in the bleak December";
//! let compressed_line = codec.compress_line(input_line);
//! assert_eq!(compressed_line, "H, dstnctly I rmmbr  t ws  n th   blk Dcmbr");
//!
//! // expand the line; check that it matches the original
//! let expanded_line = codec.expand_line(&compressed_line)?;
//! assert_eq!(input_line, expanded_line);
//!
//! // codecs also have helper methods for parsing I/O streams
//! let mut input_reader = BufReader::new(File::open("../test_data/antigonish.txt")?);
//! let mut output_writer = io::Cursor::new(Vec::new());
//! codec.compress(&mut input_reader, &mut output_writer)?;
//! let compressed_document = String::from_utf8(output_writer.into_inner())?;
//! assert_ne!("", compressed_document);
//! #    Ok(())
//! # }
//! ```

pub mod codecs;
pub mod succinct;
pub mod transcoder;

#[doc = include_str!("../README.md")]
#[cfg(doc)]
fn readme() {}