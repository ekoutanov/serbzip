use crate::succinct::Errorlike;
use crate::transcoder::{transcode, TranscodeError};
use std::convert::Infallible;
use std::error::Error;
use std::io;
use std::io::{Cursor, ErrorKind};
use crate::transcoder::TranscodeError::IoError;

#[test]
fn transcode_error_conversion() {
    let error = TranscodeError::<()>::from(io::Error::new(ErrorKind::AddrInUse, "test"));
    assert!(error.into_io_error().is_some());

    let error = TranscodeError::<()>::IoError(io::Error::new(ErrorKind::AddrInUse, "test"));
    assert_eq!(None, error.into_conversion_error());

    let error = TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    };
    assert_eq!(
        (10, Errorlike("test")),
        error.into_conversion_error().unwrap()
    );

    let error = TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    };
    assert!(error.into_io_error().is_none());
}

#[test]
fn transcode_error_into_dynamic_for_conversion_error() {
    let error = TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    };
    let boxed = error.into_dynamic();
    assert!(boxed.into_conversion_error().is_some());
}

#[test]
fn transcode_error_into_dynamic_for_io_error() {
    let error = TranscodeError::<Errorlike<String>>::IoError(io::Error::new(ErrorKind::AddrInUse, "test"));
    let boxed = error.into_dynamic();
    assert_eq!(ErrorKind::AddrInUse, boxed.into_io_error().unwrap().kind());
}

#[test]
fn transcode_error_implements_debug() {
    let error = TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    };
    let formatted = format!("{error:?}");
    assert_eq!("ConversionError { line_no: 10, error: Errorlike(\"test\") }", formatted);
}

#[test]
fn transcode_error_implements_display_for_conversion_error() {
    let error = TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    };
    let formatted = format!("{error}");
    assert_eq!("conversion error on line 10: test", formatted);
}

#[test]
fn transcode_error_implements_display_for_io_error() {
    let error = TranscodeError::<String>::IoError(io::Error::new(ErrorKind::AddrInUse, "address in use"));
    let formatted = format!("{error}");
    assert_eq!("I/O error: Custom { kind: AddrInUse, error: \"address in use\" }", formatted);
}

#[test]
fn transcode_error_implements_error() {
    let _: Box<dyn Error> = Box::new(TranscodeError::ConversionError {
        line_no: 10,
        error: Errorlike("test"),
    });
}

#[test]
fn transcode_no_errors() {
    let processor = |line_no: u32, line: &str| -> Result<String, Infallible> {
        Ok(format!("{line_no}-{line}"))
    };
    let content = concat!("first\n", "second\n", "third\n");
    let mut r = Cursor::new(content.as_bytes());
    let mut w = Cursor::new(Vec::<u8>::new());
    let result = transcode(&mut r, &mut w, processor);
    assert!(result.is_ok());
    let output = String::from_utf8(w.into_inner()).unwrap();
    let expected = concat!("1-first\n", "2-second\n", "3-third\n");
    assert_eq!(expected, output);
}

#[test]
fn transcode_with_processor_error() {
    let processor = |line_no: u32, line: &str| -> Result<String, &str> {
        match line_no {
            1 => Ok(format!("{line_no}-{line}")),
            _ => Err("could not process")
        }
    };
    let content = concat!("first\n", "second\n", "third\n");
    let mut r = Cursor::new(content.as_bytes());
    let mut w = Cursor::new(Vec::<u8>::new());
    let result = transcode(&mut r, &mut w, processor);
    assert!(result.is_err());
    assert!(matches!(result, Err(TranscodeError::ConversionError { line_no: 2, error: "could not process" })));
}

#[test]
fn transcode_with_io_error() {
    let processor = |line_no: u32, line: &str| -> Result<String, Infallible> {
        Ok(format!("{line_no}-{line}"))
    };
    let content = concat!("first\n", "second\n", "third\n");
    let mut r = Cursor::new(content.as_bytes());
    let mut w = Cursor::new([0;0]); // zero-size buffer will inhibit writes
    let result = transcode(&mut r, &mut w, processor);
    assert!(result.is_err());
    let io_error = result.err().unwrap().into_io_error().unwrap();
    assert_eq!(ErrorKind::WriteZero, io_error.kind());
}
