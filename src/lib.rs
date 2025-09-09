#![doc = include_str!("../README.md")]

#![no_std]

use core::error::Error;
use core::fmt::{Debug, Display};
use core::ops::{Deref, DerefMut};

/// View into [`str`] slice.
///
/// Holds parent `str` info which allows to safely extend this view with parent
/// size in mind.
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
///
/// view.reduce_right_while(char::is_alphabetic);
/// assert_eq!(view.as_str(), "Hello ");
/// ```
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

    pub fn as_str(&self) -> &'a str {
        &self.base[self.view_start..self.view_start + self.view_len]
    }

    /// Shrinks this view from the left to current right edge with length zero.
    ///
    /// ```toml,ignore
    /// [ str [ view ]  ]
    /// [ str    -> []  ]
    /// ```
    ///
    /// #### Example:
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.shrink_to_right();
    /// view.extend_left_while(char::is_alphabetic);
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn shrink_to_right(&mut self) {
        self.view_start += self.view_len;
        self.view_len = 0;
    }

    /// Shrinks this view from the right to current left edge with length zero.
    ///
    /// ```toml,ignore
    /// [ str [ view ]  ]
    /// [ str [] <-     ]
    /// ```
    ///
    /// #### Example:
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.shrink_to_left();
    /// view.extend_right_while(char::is_alphabetic);
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn shrink_to_left(&mut self) {
        self.view_len = 0;
    }

    /// Extend string view to the right by `n` characters.
    ///
    /// ```toml,ignore
    /// [ str  [ view ]         ]
    /// [ str  [  view  -> n ]  ]
    /// ```
    ///
    /// panics if there is not enough characters in base string to the right of this view.
    ///
    /// see [`StringView::try_extend_right`] for fallible version.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World";
    /// let mut view = text.view_part(0, 5);
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend_right(6);
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_right(&mut self, n: usize) {
        self.try_extend_right(n)
            .expect("Unable to extend string view to the right")
    }

    /// Try to extend string view to the right by `n` characters.
    ///
    /// ```toml,ignore
    /// [ str  [ view ]         ]
    /// [ str  [  view  -> n ]  ]
    /// ```
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
    /// let err = view.try_extend_right(4);
    /// assert!(err.is_ok());
    /// assert_eq!(view.as_str(), "Hello Wor");
    ///
    /// let err = view.try_extend_right(10);
    /// assert!(matches!(err, Err(BaseStringIsTooShort)));
    /// assert_eq!(view.as_str(), "Hello Wor");
    /// ```
    pub fn try_extend_right(&mut self, n: usize) -> Result<(), BaseStringIsTooShort<RIGHT>> {
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
    /// ```toml,ignore
    /// [ str  [ view ]         ]
    /// [ str  [  view  -> n ]  ]
    /// ```
    ///
    /// ### Example:
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view_part(0, 2);
    /// assert_eq!(view.as_str(), "He");
    ///
    /// view.extend_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend_right(1);
    /// view.extend_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_right_while<F>(&mut self, mut func: F)
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
    /// ```toml,ignore
    /// [ str  [   view   ]    ]
    /// [ str  [ view ] <- n   ]
    /// ```
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
    /// view.reduce_right(6);
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn reduce_right(&mut self, n: usize) {
        self.try_reduce_right(n)
            .expect("Unable to reduce string view from the right")
    }

    /// Try to reduce string view from the right by `n` characters.
    ///
    /// ```toml,ignore
    /// [ str  [   view   ]    ]
    /// [ str  [ view ] <- n   ]
    /// ```
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
    /// let result = view.try_reduce_right(6);
    /// assert!(result.is_ok());
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// let result = view.try_reduce_right(10);
    /// assert!(matches!(result, Err(ViewIsTooShort)));
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn try_reduce_right(&mut self, n: usize) -> Result<(), ViewIsTooShort<RIGHT>> {
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
    /// ```toml,ignore
    /// [ str  [   view   ]    ]
    /// [ str  [ view ] <- n   ]
    /// ```
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello World !!!";
    ///
    /// let mut view = text.view();
    /// assert_eq!(view.as_str(), "Hello World !!!");
    ///
    /// view.reduce_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World ");
    ///
    /// view.reduce_right(1);
    /// view.reduce_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello ");
    /// ```
    pub fn reduce_right_while<F>(&mut self, mut func: F)
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
    /// ```toml,ignore
    /// [ str        [ view ]  ]
    /// [ str  [ n <- view  ]  ]
    /// ```
    ///
    /// panics if there is not enough characters in base string to the left of this view.
    ///
    /// see [`StringView::try_extend_left`] for fallible version.
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
    /// ```toml,ignore
    /// [ str        [ view ]  ]
    /// [ str  [ n <- view  ]  ]
    /// ```
    ///
    /// returns [`Err`] if there is not enough characters in base string to the right of this view.
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
    /// ```toml,ignore
    /// [ str        [ view ]  ]
    /// [ str  [ n <- view  ]  ]
    /// ```
    ///
    /// #### Example:
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
    /// ```toml,ignore
    /// [ str   [   view   ]   ]
    /// [ str  n -> [ view ]   ]
    /// ```
    ///
    /// panics if there is not enough characters in current string view.
    ///
    /// see [`StringView::try_reduce_left`] for fallible version.
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
    /// ```toml,ignore
    /// [ str   [   view   ]   ]
    /// [ str  n -> [ view ]   ]
    /// ```
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
    /// ```toml,ignore
    /// [ str   [   view   ]   ]
    /// [ str  n -> [ view ]   ]
    /// ```
    ///
    /// #### Example:
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

/// The only error case in [`StringView::try_extend_right`].
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

/// The only error case in [`StringView::try_reduce_right`].
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
        Char(self.0)
    }

    pub fn is_same_size(&self, ch: char) -> bool {
        self.char().len_utf8() == ch.len_utf8()
    }

    pub fn replace(&mut self, ch: char) -> Result<(), CharsHaveDifferentSizes> {
        if !self.is_same_size(ch) {
            return Err(CharsHaveDifferentSizes);
        }
        let mut buf: [u8; 4] = [0; 4];
        let subslice: &[u8] = ch.encode_utf8(&mut buf).as_bytes();

        // Safety: self and subslice have the same number of bytes
        unsafe {
            for (idx, byte) in self.as_bytes_mut().iter_mut().enumerate() {
                *byte = subslice[idx];
            }
        }
        Ok(())
    }

    /// Makes [`CharMut`] uppercase in-place.
    ///
    /// returns [`Err`] if uppercase variant has different size.
    ///
    /// ```rust
    /// # extern crate std;
    /// # use std::string::String;
    ///
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("Hello World");
    /// text.chars_in_place_mut().for_each(|mut ch| ch.make_uppercase().unwrap());
    ///
    /// assert_eq!(text, "HELLO WORLD");
    /// ```
    pub fn make_uppercase(&mut self) -> Result<(), CharsHaveDifferentSizes> {
        let this_char = self.char();
        let mut upper_chars = this_char.to_uppercase();
        let this_upper = upper_chars.next().unwrap();

        if upper_chars.next().is_some() {
            return Err(CharsHaveDifferentSizes);
        };
        self.replace(this_upper)
    }

    /// Makes [`CharMut`] uppercase in-place.
    ///
    /// returns [`Err`] if uppercase variant has different size.
    ///
    /// ```rust
    /// # extern crate std;
    /// # use std::string::String;
    ///
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("Hello World");
    /// text.chars_in_place_mut().for_each(|mut ch| ch.make_lowercase().unwrap());
    ///
    /// assert_eq!(text, "hello world");
    /// ```
    pub fn make_lowercase(&mut self) -> Result<(), CharsHaveDifferentSizes> {
        let this_char = self.char();
        let mut lower_chars = this_char.to_lowercase();
        let this_lower = lower_chars.next().unwrap();

        if lower_chars.next().is_some() {
            return Err(CharsHaveDifferentSizes);
        };
        self.replace(this_lower)
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

/// Common error case while working with chars in-place
pub struct CharsHaveDifferentSizes;

impl Debug for CharsHaveDifferentSizes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Unable to replace character because they have different sizes.\
            Characters have to have the same size for in-place modification."
        )
    }
}

impl Display for CharsHaveDifferentSizes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Error for CharsHaveDifferentSizes {}

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

    /// Makes str characters lowercase in-place where appropriate
    fn make_lowercase(&mut self);

    /// Makes str characters uppercase in-place where appropriate
    fn make_uppercase(&mut self);
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

    /// Makes [`str`] characters lowercase in-place where appropriate
    ///
    /// ```rust
    /// # extern crate std;
    /// # use std::string::String;
    ///
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("HELLO World");
    /// text.make_lowercase();
    /// assert_eq!(text, "hello world");
    ///
    /// let text: &mut str = &mut String::from("ПРИВЕТ Мир");
    /// text.make_lowercase();
    /// assert_eq!(text, "привет мир");
    /// ```
    fn make_lowercase(&mut self) {
        self.chars_in_place_mut().for_each(|mut ch| {
            let _ = ch.make_lowercase();
        });
    }

    /// Makes [`str`] characters uppercase in-place where appropriate
    ///
    /// ```rust
    /// # extern crate std;
    /// # use std::string::String;
    ///
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("Hello World");
    /// text.make_uppercase();
    /// assert_eq!(text, "HELLO WORLD");
    ///
    /// let text: &mut str = &mut String::from("Привет мир");
    /// text.make_uppercase();
    /// assert_eq!(text, "ПРИВЕТ МИР");
    /// ```
    fn make_uppercase(&mut self) {
        self.chars_in_place_mut().for_each(|mut ch| {
            let _ = ch.make_uppercase();
        });
    }
}

#[cfg(test)]
mod test;
