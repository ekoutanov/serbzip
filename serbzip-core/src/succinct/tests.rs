use std::any::TypeId;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use crate::succinct::{CowStr, Errorlike};

// $coverage:ignore-start

#[test]
fn errorlike_from_owned() {
    let errorlike = Errorlike::<CowStr>::owned(String::from("test"));
    assert_eq!("test", errorlike.0.as_ref());
    assert_eq!(String::from("test"), errorlike.0.into_owned());
}

#[test]
fn errorlike_from_borrowed() {
    let errorlike = Errorlike::<CowStr>::borrowed("test");
    assert_eq!("test", errorlike.0.as_ref());
    assert_eq!(String::from("test"), errorlike.0.into_owned());
}

#[test]
fn errorlike_implements_debug() {
    #[derive(Debug)]
    struct Test;
    let errorlike = Errorlike(Test);
    assert_eq!(String::from("Errorlike(Test)"), format!("{errorlike:?}"));
}

#[test]
fn errorlike_implements_display() {
    #[derive(Debug)]
    struct Test;

    impl Display for Test {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "nice Test")
        }
    }

    let errorlike = Errorlike(Test);
    assert_eq!(String::from("nice Test"), format!("{errorlike}"));
}

#[test]
fn errorlike_implements_error() {
    let errorlike = Errorlike::<CowStr>::borrowed("test");
    let box_of_errorlike = Box::new(errorlike);
    assert_eq!(
        TypeId::of::<Errorlike<Cow<'static, str>>>(),
        core::any::Any::type_id(box_of_errorlike.deref())
    );

    let box_of_error: Box<dyn Error> = box_of_errorlike;
    assert_eq!(
        TypeId::of::<dyn Error>(),
        core::any::Any::type_id(box_of_error.deref())
    );
}

// $coverage:ignore-end
