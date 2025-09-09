extern crate std;
use std::prelude::rust_2021::*;

use string_view::StrExt;

use crate as string_view;

#[test]
fn chars_in_place() {
    let mut iter = "Hello".chars_in_place();

    assert_eq!(*"H", *iter.next().unwrap());
    assert_eq!(*"e", *iter.next().unwrap());
    assert_eq!(*"l", *iter.next().unwrap());
    assert_eq!(*"l", *iter.next().unwrap());
    assert_eq!(*"o", *iter.next().unwrap());
}

#[test]
fn chars_in_place_mut() {
    let text: &mut str = &mut String::from("Hello");

    let mut iter = text.chars_in_place_mut();

    assert_eq!(*"H", *iter.next().unwrap());
    assert_eq!(*"e", *iter.next().unwrap());
    assert_eq!(*"l", *iter.next().unwrap());
    assert_eq!(*"l", *iter.next().unwrap());
    assert_eq!(*"o", *iter.next().unwrap());
}