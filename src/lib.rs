//! A lightweight framework for transcoding text from one lexical form to another.
//!
//! The apparatus that performs this trancoding is called a **codec**. It is described by the
//! [Codec](crate::codecs::Codec) trait. A prominent example of such a codec is **Balkanoid** —
//! a quasi-lossless Balkanoidal meta-lingual compressor.
//!
//! This crate provides mechanisms for creating codecs, as well as one or more useful
//! codec implementations.

pub mod codecs;
pub mod succinct;
pub mod transcoder;
