use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct Errorlike<T>(pub T);

impl<'a, C> Errorlike<Cow<'a, C>> where C: ?Sized + 'a + ToOwned {
    pub fn from_owned(c: <C as ToOwned>::Owned) -> Self {
        Self(Cow::Owned(c))
    }

    pub fn from_borrowed(c: &'a C) -> Self {
        Self(Cow::Borrowed(c))
    }
}

impl<T: Display + Debug> Display for Errorlike<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Display + Debug> Error for Errorlike<T> {}

pub type CowStr = Cow<'static, str>;

pub trait Stringlike : AsRef<str> {
    fn into_owned(self) -> String;
}

impl Stringlike for CowStr {
    fn into_owned(self) -> String {
        self.into_owned()
    }
}

impl Stringlike for String {
    fn into_owned(self) -> String {
        self
    }
}

// impl Stringlike for String {
//     fn to_owned(self) -> String {
//         self
//     }
//
//     fn to_borrowed(&self) -> &str {
//         self
//     }
// }
//
// impl Stringlike for CowStr {
//     fn to_owned(self) -> String {
//         self.into_owned()
//     }
//
//     fn to_borrowed(&self) -> &str {
//         let g = self;
//         CowStr::to_borrowed(g)
//     }
// }

#[derive(Debug)]
pub enum IoElseErrorlike<T> {
    IoError(io::Error),
    Errorlike(T)
}

impl <T> From<io::Error> for IoElseErrorlike<T> {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl <T> From<Errorlike<T>> for IoElseErrorlike<T> {
    fn from(error: Errorlike<T>) -> Self {
        Self::Errorlike(error.0)
    }
}

impl<T: Display> Display for IoElseErrorlike<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IoElseErrorlike::IoError(error) => Debug::fmt(error, f),
            IoElseErrorlike::Errorlike(error) => Display::fmt(error, f)
        }
    }
}

impl <T: Debug + Display> Error for IoElseErrorlike<T> {}