use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Errorlike<T>(pub T);

pub type CowStr = Cow<'static, str>;

// impl <T> Monad<T> {
//     pub fn boxify(self) -> Box<Self> {
//         Box::new(self)
//     }
// }

// impl<T: Debug + Display> Monad<T> {
//     pub fn from(t: T) -> Self {
//         Self(t)
//     }
// }

// impl Monad<CowString> {
//     pub fn from_owned(c: String) -> Self {
//         Self(Cow::Owned(c))
//     }
//
//     pub fn from_borrowed(c: &'static str) -> Self {
//         Self(Cow::Borrowed(c))
//     }
// }
//
// pub enum Gow<'a, B>
//     where
//         B: ToOwned + 'a,
// {
//     /// Borrowed data.
//     Borrowed(&'a B),
//
//     /// Owned data.
//     Owned(<B as ToOwned>::Owned),
// }

impl<'a, C> Errorlike<Cow<'a, C>> where C: ?Sized + 'a + ToOwned {
    pub fn from_owned(c: <C as ToOwned>::Owned) -> Self {
        Self(Cow::Owned(c))
    }

    pub fn from_borrowed(c: &'a C) -> Self {
        Self(Cow::Borrowed(c))
    }
}

// impl<C> Monad<Gow<'static, C>> where C: ?Sized + 'static {
//     pub fn from_owned(c: <C as ToOwned>::Owned) -> Self {
//         Self(Gow::Owned(c))
//     }
//
//     pub fn from_borrowed(c: &'static C) -> Self {
//         Self(Gow::Borrowed(c))
//     }
// }

impl<T: Display + Debug> Display for Errorlike<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Display + Debug> Error for Errorlike<T> {}

// impl <T> Into<Box<T>> for Monad<T> {
//     fn into(self) -> Box<T> {
//         Box::new(self)
//     }
// }
