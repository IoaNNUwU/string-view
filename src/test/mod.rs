extern crate std;
use std::prelude::rust_2021::*;

use string_view::StrExt;

use crate as string_view;

#[test]
fn chars_in_place() {
    let mut iter = "Hello".chars_in_place();

    assert_eq!(iter.next().unwrap(), "H");
    assert_eq!(iter.next().unwrap(), "e");
    assert_eq!(iter.next().unwrap(), "l");
    assert_eq!(iter.next().unwrap(), 'l');
    assert_eq!(iter.next().unwrap(), 'o');
}

#[test]
fn chars_in_place_mut() {
    let text: &mut str = &mut String::from("Hello");

    let mut iter = text.chars_in_place_mut();

    assert_eq!(iter.next().unwrap(), "H");
    assert_eq!(iter.next().unwrap(), "e");
    assert_eq!(iter.next().unwrap(), "l");
    assert_eq!(iter.next().unwrap(), 'l');
    assert_eq!(iter.next().unwrap(), 'o');
}

#[test]
fn chars_in_place_rev() {
    let mut iter = "Hello".chars_in_place().rev();

    assert_eq!(iter.next().unwrap(), 'o');
    assert_eq!(iter.next().unwrap(), 'l');
    assert_eq!(iter.next().unwrap(), "l");
    assert_eq!(iter.next().unwrap(), "e");
    assert_eq!(iter.next().unwrap(), "H");
}

#[test]
fn chars_in_place_mut_rev() {
    let text: &mut str = &mut String::from("Hello");

    let mut iter = text.chars_in_place_mut().rev();

    assert_eq!(iter.next().unwrap(), 'o');
    assert_eq!(iter.next().unwrap(), 'l');
    assert_eq!(iter.next().unwrap(), "l");
    assert_eq!(iter.next().unwrap(), "e");
    assert_eq!(iter.next().unwrap(), "H");
}
