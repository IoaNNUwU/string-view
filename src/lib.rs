//! ## String View
//!
//! #### Work with views into string slices. Safely extend, reduce without losing parent string size.
//!
//! #### Use in-place modifications to speed up your code.
//!
//! Example:
//!
//! ```rust
//! let program_text = r#"
//! fn main() {
//!     let text = "Hello World";
//! }
//! "#;
//!
//! use string_view::StrExt;
//!
//! let mut view = program_text.view_part(0, 0);
//! view.extend_while(|ch| ch == ' ' || ch == '\n');
//! view.extend_while(char::is_alphabetic);
//! view.reduce_left_while(|ch| ch == ' ' || ch == '\n');
//! assert_eq!(view.as_str(), "fn");
//!
//! view.try_extend(1).unwrap();
//! view.extend_while(char::is_alphabetic);
//! view.try_extend(2).unwrap();
//! assert_eq!(view.as_str(), "fn main()");
//!
//! view.extend_while(|ch| ch == ' ' || ch == '\n' || ch == '{');
//! view.shrink_to_end();
//! view.extend_while(|_| true);
//! view.reduce_while(|ch| ch == ' ' || ch == '\n' || ch == '}');
//! assert_eq!(view.as_str(), r#"let text = "Hello World";"#);
//! 
//! view.reduce_while(|ch| ch == ';');
//! view.reduce(1);
//! view.shrink_to_end();
//! view.extend_left_while(|ch| ch != '"');
//! assert_eq!(view.as_str(), "Hello World");
//! ```

#![no_std]

use core::error::Error;
use core::fmt::{Debug, Display};
use core::ops::{Deref, DerefMut};

pub struct StringView<'a> {
    pub base: &'a str,
    /// byte idx of view start inside [`StringView::base`].
    view_start: usize,
    /// byte len of view inside [`StringView::base`].
    view_len: usize,
}

impl<'a> StringView<'a> {
    /// Creates [`StringView`] of a whole [`str`] slice.
    ///
    /// see [`StringView::new_part`] to view part of [`str`] slice.
    pub fn new(base: &'a str) -> Self {
        Self {
            base,
            view_start: 0,
            view_len: base.len(),
        }
    }

    /// Creates [`StringView`] of a part of [`str`] slice using 2 byte indexes.
    ///
    /// ```rust
    /// use string_view::StringView;
    ///
    /// let text = "Hello World";
    /// let view = StringView::new_part(text, 6, 11);
    ///
    /// assert_eq!(view.as_str(), "World");
    /// ```
    ///
    /// Or using [`StrExt`] extension trait:
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let view = text.view_part(6, 11);
    ///
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn new_part(base: &'a str, view_start: usize, view_end: usize) -> Self {
        assert!(
            view_end >= view_start,
            "View end index cannot be less then start index"
        );
        Self {
            base,
            view_start,
            view_len: view_end - view_start,
        }
    }

    /// Byte index of view start inside [`StringView::base`].
    pub fn start(&self) -> usize {
        self.view_start
    }

    /// Byte index of view end inside [`StringView::base`].
    pub fn end(&self) -> usize {
        self.view_start + self.view_len
    }

    pub fn as_str(&self) -> &str {
        &self.base[self.view_start..self.view_start + self.view_len]
    }

    pub fn shrink_to_end(&mut self) {
        self.view_start += self.view_len;
        self.view_len = 0;
    }

    pub fn shrink_to_start(&mut self) {
        self.view_len = 0;
    }

    /// Extend string view to the right by `n` characters.
    ///
    /// panics if there is not enough characters in base string to the right of this view.
    ///
    /// see [`StringView::try_extend`] for fallible version.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut view = text.view_part(0, 5);
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend(6);
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend(&mut self, n: usize) {
        self.try_extend(n)
            .expect("Unable to extend string view to the right")
    }

    /// Try to extend string view to the right by `n` characters.
    ///
    /// returns [`Err`] if there is not enough characters in base string to the right of this view.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    ///
    /// let mut view = text.view_part(0, 5);
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// let err = view.try_extend(4);
    /// assert!(err.is_ok());
    /// assert_eq!(view.as_str(), "Hello Wor");
    ///
    /// let err = view.try_extend(10);
    /// assert!(matches!(err, Err(BaseStringIsTooShort)));
    /// assert_eq!(view.as_str(), "Hello Wor");
    /// ```
    pub fn try_extend(&mut self, n: usize) -> Result<(), BaseStringIsTooShort<RIGHT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base[self.end()..].chars();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(BaseStringIsTooShort)?.len_utf8();
        }
        self.view_len += combined_len;
        Ok(())
    }

    /// Extend string view to the right while `func` returns `true`.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view_part(0, 2);
    /// assert_eq!(view.as_str(), "He");
    ///
    /// view.extend_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend(1);
    /// view.extend_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;

        for ch in self.base[self.end()..].chars() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_len += combined_len;
    }

    /// Reduce string view from the right by `n` characters.
    ///
    /// panics if there is not enough characters in current string view.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    ///
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.reduce(6);
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn reduce(&mut self, n: usize) {
        self.try_reduce(n)
            .expect("Unable to reduce string view from the right")
    }

    /// Try to reduce string view from the right by `n` characters.
    ///
    /// returns [`Err`] if there is not enough characters in current string view.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "One and only Hello World";
    ///
    /// let mut view = text.view_part(13, 24);
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// let result = view.try_reduce(6);
    /// assert!(result.is_ok());
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// let result = view.try_reduce(10);
    /// assert!(matches!(result, Err(ViewIsTooShort)));
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn try_reduce(&mut self, n: usize) -> Result<(), ViewIsTooShort<RIGHT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base[self.start()..self.end()].chars().rev();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(ViewIsTooShort)?.len_utf8();
        }
        self.view_len -= combined_len;
        Ok(())
    }

    /// Reduce string view from the right while `func` returns `true`.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World !!!");
    ///
    /// view.reduce_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World ");
    ///
    /// view.reduce(1);
    /// view.reduce_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello ");
    /// ```
    pub fn reduce_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;
        for ch in self.base[self.start()..self.end()].chars().rev() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_len -= combined_len;
    }

    /// Extend string view to the left by `n` characters.
    ///
    /// panics if there is not enough characters in base string to the left of this view.
    ///
    /// see [`StringView::try_extend_left`] for fallible version.
    ///
    /// see [`StringView::extend`] to extend to the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut view = text.view_part(6, 11);
    /// assert_eq!(view.as_str(), "World");
    ///
    /// view.extend_left(6);
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_left(&mut self, n: usize) {
        self.try_extend_left(n)
            .expect("Unable to extend string view to the left")
    }

    /// Try to extend string view to the left by `n` characters.
    ///
    /// returns [`Err`] if there is not enough characters in base string to the right of this view.
    ///
    /// see [`StringView::try_extend`] to extend to the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    ///
    /// let mut view = text.view_part(6, 11);
    /// assert_eq!(view.as_str(), "World");
    ///
    /// let err = view.try_extend_left(4);
    /// assert!(err.is_ok());
    /// assert_eq!(view.as_str(), "llo World");
    ///
    /// let err = view.try_extend_left(10);
    /// assert!(matches!(err, Err(BaseStringIsTooShort)));
    /// assert_eq!(view.as_str(), "llo World");
    /// ```
    pub fn try_extend_left(&mut self, n: usize) -> Result<(), BaseStringIsTooShort<LEFT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base[..self.start()].chars().rev();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(BaseStringIsTooShort)?.len_utf8();
        }
        self.view_start -= combined_len;
        self.view_len += combined_len;
        Ok(())
    }

    /// Extend string view to the left while `func` returns `true`.
    ///
    /// see [`StringView::extend_while`] to extend to the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view_part(14, 15);
    /// assert_eq!(view.as_str(), "!");
    ///
    /// view.extend_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "!!!");
    ///
    /// view.extend_left(1);
    /// view.extend_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "World !!!");
    /// ```
    pub fn extend_left_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;

        for ch in self.base[..self.start()].chars().rev() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_start -= combined_len;
        self.view_len += combined_len;
    }

    /// Reduce string view from the left by `n` characters.
    ///
    /// panics if there is not enough characters in current string view.
    ///
    /// see [`StringView::try_reduce_left`] for fallible version.
    ///
    /// see [`StringView::reduce`] to reduce from the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    ///
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.reduce_left(6);
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn reduce_left(&mut self, n: usize) {
        self.try_reduce_left(n)
            .expect("Unable to reduce string view from the left")
    }

    /// Try to reduce string view from the left by `n` characters.
    ///
    /// returns [`Err`] if there is not enough characters in current string view.
    ///
    /// see [`StringView::try_reduce`] to reduce from the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "One and only Hello World";
    ///
    /// let mut view = text.view_part(13, 24);
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// let result = view.try_reduce_left(6);
    /// assert!(result.is_ok());
    /// assert_eq!(view.as_str(), "World");
    ///
    /// let result = view.try_reduce_left(10);
    /// assert!(matches!(result, Err(ViewIsTooShort)));
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn try_reduce_left(&mut self, n: usize) -> Result<(), ViewIsTooShort<LEFT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base[self.start()..self.end()].chars();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(ViewIsTooShort)?.len_utf8();
        }
        self.view_start += combined_len;
        self.view_len -= combined_len;
        Ok(())
    }

    /// Reduce string view from the left while `func` returns `true`.
    ///
    /// see [`StringView::reduce_while`] to reduce from the right.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World !!!");
    ///
    /// view.reduce_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), " World !!!");
    ///
    /// view.reduce_left(1);
    /// view.reduce_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), " !!!");
    /// ```
    pub fn reduce_left_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;
        for ch in self.base[self.start()..self.end()].chars() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_start += combined_len;
        self.view_len -= combined_len;
    }
}

impl Deref for StringView<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Debug for StringView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StringView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

type Side = bool;
const RIGHT: bool = true;
const LEFT: bool = false;

/// The only error case in [`StringView::try_extend`].
pub struct BaseStringIsTooShort<const SIDE: Side>;

impl<const SIDE: Side> Debug for BaseStringIsTooShort<SIDE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Base String contains less characters than `n` to the {} of the view",
            if SIDE == RIGHT { "right" } else { "left" }
        )
    }
}

impl<const SIDE: Side> Display for BaseStringIsTooShort<SIDE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<const SIDE: Side> Error for BaseStringIsTooShort<SIDE> {}

/// The only error case in [`StringView::try_reduce`].
pub struct ViewIsTooShort<const SIDE: Side>;

impl<const SIDE: Side> Debug for ViewIsTooShort<SIDE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "View contains less characters than `n` to the {} of the view",
            if SIDE == RIGHT { "right" } else { "left" }
        )
    }
}

impl<const SIDE: Side> Display for ViewIsTooShort<SIDE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<const SIDE: Side> Error for ViewIsTooShort<SIDE> {}

/// In-place character representation inside str slice
///
/// Under the hood is &str with single character
///
/// ```rust
/// use string_view::Char;
///
/// let ch = Char::new("A");
/// ```
///
/// ```rust,should_panic
/// use string_view::Char;
///
/// let ch = Char::new(""); // panics
/// let ch = Char::new("Hello"); // panics
/// ```
pub struct Char<'a>(&'a str);

impl Char<'_> {
    pub fn new(ch: &str) -> Char<'_> {
        let char_len = ch
            .chars()
            .next()
            .expect("Unable to create Char from empty string")
            .len_utf8();

        assert_eq!(
            char_len,
            ch.len(),
            "Char can only be constructed from single character string"
        );

        Char(ch)
    }

    pub fn char(&self) -> char {
        self.0.chars().next().unwrap()
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl<'a> Deref for Char<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.0
    }
}

impl Debug for Char<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Char<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Iterator of chars in-place
pub struct CharsInPlace<'a>(&'a str);

impl<'a> Iterator for CharsInPlace<'a> {
    type Item = Char<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_char_len = self.0.chars().next()?.len_utf8();

        let (this, rest) = self.0.split_at(next_char_len);
        self.0 = rest;

        Some(Char(this))
    }
}

/// In-place character representation inside mutable str slice
///
/// Convert to [`Char`] using [`CharMut::as_char`].
pub struct CharMut<'a>(&'a mut str);

impl CharMut<'_> {
    pub fn new(ch: &mut str) -> CharMut<'_> {
        let char_len = ch
            .chars()
            .next()
            .expect("Unable to create CharMut from empty string")
            .len_utf8();

        assert_eq!(
            char_len,
            ch.len(),
            "CharMut can only be constructed from single character string"
        );

        CharMut(ch)
    }

    pub fn char(&self) -> char {
        self.0.chars().next().unwrap()
    }

    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn as_str_mut(&mut self) -> &mut str {
        self.0
    }

    pub fn as_char(&self) -> Char<'_> {
        Char(&self.0)
    }
}

impl<'a> Deref for CharMut<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.0
    }
}

impl<'a> DerefMut for CharMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Debug for CharMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for CharMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Mutable iterator of chars in-place
pub struct CharsInPlaceMut<'a>(&'a mut str);

impl<'a> Iterator for CharsInPlaceMut<'a> {
    type Item = CharMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_char_len = self.0.chars().next()?.len_utf8();

        let this: &mut str = core::mem::take(&mut self.0);
        let (this, rest) = this.split_at_mut(next_char_len);
        self.0 = rest;

        Some(CharMut(this))
    }
}

pub trait StrExt {
    fn view(&self) -> StringView<'_>;
    fn view_part(&self, start_idx: usize, end_idx: usize) -> StringView<'_>;

    /// Byte index of this [`Char`] start & end inside base [`str`].
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut chars = text.chars_in_place();
    ///
    /// let first_char = chars.next().unwrap();
    /// assert_eq!(first_char.as_str(), "H");
    /// assert_eq!(text.char_idx(first_char), (0, 1));
    ///
    /// let second_char = chars.next().unwrap();
    /// assert_eq!(second_char.as_str(), "e");
    /// assert_eq!(text.char_idx(second_char), (1, 2));
    /// ```
    fn char_idx(&self, ch: Char) -> (usize, usize);

    fn chars_in_place(&self) -> CharsInPlace<'_>;
    fn chars_in_place_mut(&mut self) -> CharsInPlaceMut<'_>;
}

impl StrExt for str {
    fn view(&self) -> StringView<'_> {
        StringView::new(self)
    }
    fn view_part(&self, start_idx: usize, end_idx: usize) -> StringView<'_> {
        StringView::new_part(self, start_idx, end_idx)
    }
    fn chars_in_place(&self) -> CharsInPlace<'_> {
        CharsInPlace(self)
    }
    fn chars_in_place_mut(&mut self) -> CharsInPlaceMut<'_> {
        CharsInPlaceMut(self)
    }

    /// Byte index of this [`Char`] start & end inside base [`str`].
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut chars = text.chars_in_place();
    ///
    /// let first_char = chars.next().unwrap();
    /// assert_eq!(first_char.as_str(), "H");
    /// assert_eq!(text.char_idx(first_char), (0, 1));
    ///
    /// let second_char = chars.next().unwrap();
    /// assert_eq!(second_char.as_str(), "e");
    /// assert_eq!(text.char_idx(second_char), (1, 2));
    /// ```
    fn char_idx(&self, ch: Char) -> (usize, usize) {
        let str_start = self.as_ptr() as usize;
        let str_end = str_start + self.len();
        let ch_start = ch.as_ptr() as usize;
        let ch_end = ch_start + ch.len();

        let range = str_start..str_end;
        assert!(
            range.contains(&ch_start),
            "Char has to be inside this string to get its index"
        );
        assert!(
            range.contains(&ch_end),
            "Char has to be inside this string to get its index"
        );

        (ch_start - str_start, ch_end - str_start)
    }
}

#[cfg(test)]
mod test;
