//! Various types and helpers that reduce boilerplate code.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errorlike<T>(pub T);

impl<'a, C> Errorlike<Cow<'a, C>>
where
    C: ?Sized + 'a + ToOwned,
{
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

pub trait Stringlike: AsRef<str> {
    fn into_owned(self) -> String;
}

impl<'a> Stringlike for Cow<'a, str> {
    fn into_owned(self) -> String {
        self.into_owned()
    }
}

impl Stringlike for String {
    fn into_owned(self) -> String {
        self
    }
}

impl Stringlike for &str {
    fn into_owned(self) -> String {
        String::from(self)
    }
}

#[cfg(test)]
mod tests;
