//! Various types and helpers that reduce boilerplate code.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// [`Errorlike`] is a newtype for conditionally implementing the [`Error`] trait on types that
/// satisfy [`Debug`] and [`Display`] but do not implement [`Error`] directly.
///
/// This is used when you need to return an [`Error`], but don't have one handy.
///
/// # Examples
/// ```
/// use serbzip::succinct::Errorlike;
/// use std::error::Error;
/// use serbzip::succinct::CowStr;
///
/// # fn something_wrong() -> bool {
/// #     false
/// # }
/// #
/// fn main() -> Result<(), Box<dyn Error>> {
///     if something_wrong() {
///         return Err(Errorlike("something awful just happened"))?;
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errorlike<T>(pub T);

impl<'a, C> Errorlike<Cow<'a, C>>
where
    C: ?Sized + 'a + ToOwned,
{
    /// Convenience for constructing an [`Errorlike`] encapsulating a [`Cow`] that contains
    /// owned data.
    pub fn owned(c: <C as ToOwned>::Owned) -> Self {
        Self(Cow::Owned(c))
    }

    /// Convenience for constructing an [`Errorlike`] encapsulating a [`Cow`] that contains
    /// borrowed data.
    pub fn borrowed(c: &'a C) -> Self {
        Self(Cow::Borrowed(c))
    }
}

impl<T: Display + Debug> Display for Errorlike<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Display + Debug> Error for Errorlike<T> {}

/// An alias for a very common type of [`Cow`], being a lazily constructed [`String`] from
/// a `'static` string slice.
pub type CowStr = Cow<'static, str>;

// /// Something that can pass for a [`String`] or a [`&str`].
// pub trait Stringlike: AsRef<str> + Into<String> {
//     fn into_owned(self) -> String;
// }
//
// impl<'a> Stringlike for Cow<'a, str> {
//     fn into_owned(self) -> String {
//         self.into_owned()
//     }
// }
//
// impl Stringlike for String {
//     fn into_owned(self) -> String {
//         self
//     }
// }
//
// impl Stringlike for &str {
//     fn into_owned(self) -> String {
//         String::from(self)
//     }
// }

#[cfg(test)]
mod tests;
