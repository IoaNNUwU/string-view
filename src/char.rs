use core::error::Error;
use core::fmt::{Debug, Display};

/// In-place character representation inside string slice.
///
/// Under the hood `Char` is `&str` pointing to single character inside base string.
///
/// ```rust
/// use string_view::Char;
///
/// let ch = Char::new("A");
/// let ch = Char::new("日");
///
/// let ch = Char::new(&"Hello World"[3..4]);
/// assert_eq!(ch, "l");
/// ```
///
/// ```rust,should_panic
/// use string_view::Char;
///
/// let ch = Char::new(""); // panics
/// let ch = Char::new("Hello"); // panics
///
/// let ch = Char::new(&"Hello World"[3..6]); // panics
/// ```
#[derive(PartialEq, Eq)]
pub struct Char<'a>(&'a str);

impl Char<'_> {
    /// Creates new `Char` from single-character string slice. This character can take
    /// from 1 to 4 bytes inside string slice.
    ///
    /// **Panics** if argument is not single-character string slice.
    pub fn new(ch: &str) -> Char<'_> {
        let char_len = ch
            .chars()
            .next()
            .expect("Unable to create Char from empty string")
            .len_utf8();

        assert_eq!(
            char_len,
            ch.len(),
            "Char can only be constructed from single character string slice"
        );

        Char(ch)
    }

    pub fn char(&self) -> char {
        // SAFETY: Char always points to string of at least 1 character.
        // Chars created from CharMut can point to more than one, but never 0.
        unsafe { self.as_str().chars().next().unwrap_unchecked() }
    }

    pub fn as_str(&self) -> &str {
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

impl PartialEq<char> for Char<'_> {
    fn eq(&self, other: &char) -> bool {
        self.char() == *other
    }
}

impl PartialEq<&str> for Char<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Immutable iterator of chars in-place.
///
/// ```rust
/// use string_view::{CharsInPlace, Char};
///
/// let text = "Hello";
///
/// let mut chars = CharsInPlace::new(text);
///
/// assert_eq!(chars.next().unwrap(), "H");
/// assert_eq!(chars.next().unwrap(), "e");
/// assert_eq!(chars.next().unwrap(), 'l');
/// assert_eq!(chars.next().unwrap(), 'l');
/// assert_eq!(chars.next().unwrap(), 'o');
/// ```
///
/// See [`CharsInPlaceMut`] for mutable version.
/// See [`StrExt::chars_in_place`](crate::StrExt::chars_in_place) for method syntax.
pub struct CharsInPlace<'a>(&'a str);

impl<'a> CharsInPlace<'a> {
    pub fn new(s: &'a str) -> Self {
        CharsInPlace(s)
    }
}

impl<'a> Iterator for CharsInPlace<'a> {
    type Item = Char<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_char_len = self.0.chars().next()?.len_utf8();

        let (this, rest) = self.0.split_at(next_char_len);
        self.0 = rest;

        Some(Char(this))
    }
}

impl<'a> DoubleEndedIterator for CharsInPlace<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_char_len = self.0.chars().rev().next()?.len_utf8();

        let (rest, this) = self.0.split_at(self.0.len() - next_char_len);
        self.0 = rest;

        Some(Char(this))
    }
}

/// In-place character representation inside mutable str slice
///
/// Convert to [`Char`] using [`CharMut::as_char`].
///
/// Under the hood is `&mut str` with single character
///
/// ```rust
/// use string_view::StrExt;
///
/// let mut text = String::from("Hello");
///
/// for mut char in text.chars_in_place_mut() {
///     char.make_uppercase();
/// }
/// assert_eq!(text, "HELLO");
///
/// text.chars_in_place_mut().nth(2).unwrap().replace('-');
/// assert_eq!(text, "HE-LO");
/// ```
#[derive(PartialEq, Eq)]
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

    /// Extract this character from string
    pub fn char(&self) -> char {
        // Char makes use of unsafe to make this faster
        // CharMut -> Char conversion is noop
        self.as_char().char()
    }

    /// Get underlaying string slice
    pub fn as_str(&self) -> &str {
        self.0
    }

    /// Get underlaying mutable string slice.
    ///
    /// This function generally **shouldn't be used** as you are able to mutate this slice
    /// in a way that breaks [`CharMut`]s guarantees. For example you can replace 1
    /// multiple-byte character with multiple single-byte characters. This won't break
    /// invariants of the string slice, but will for [`CharMut`] as it has to point to
    /// single character string.
    ///
    /// This function is safe because broken invariants inside [`CharMut`] cannot cause
    /// undefined behavior, just wrong output. For example, in this case [`.char()`](CharMut::char)
    /// will return first character it is able to find, not an original string slice.
    pub fn as_str_mut(&mut self) -> &mut str {
        self.0
    }

    /// Convert [`CharMut`] to [`Char`].
    ///
    /// Use [`.char()`](CharMut::char) for `char` conversion.
    pub fn as_char(&self) -> Char<'_> {
        Char(self.0)
    }

    pub fn is_same_size(&self, ch: char) -> bool {
        self.0.len() == ch.len_utf8()
    }

    /// Replace character with new one in-place.
    ///
    /// Checks at runtime if chars have the same length in `UTF-8` and returns an error if they don't.
    pub fn replace(&mut self, ch: char) -> Result<(), CharsHaveDifferentSizes> {
        if !self.is_same_size(ch) {
            return Err(CharsHaveDifferentSizes);
        }
        let mut buf: [u8; 4] = [0; 4];
        let subslice: &[u8] = ch.encode_utf8(&mut buf).as_bytes();

        // Safety: self and subslice have the same number of bytes
        unsafe {
            for (idx, byte) in self.as_str_mut().as_bytes_mut().iter_mut().enumerate() {
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

impl PartialEq<char> for CharMut<'_> {
    fn eq(&self, other: &char) -> bool {
        self.char() == *other
    }
}

impl PartialEq<&str> for CharMut<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Mutable iterator of chars in-place
///
/// See [`CharsInPlace`] for immutable version
pub struct CharsInPlaceMut<'a>(&'a mut str);

impl<'a> CharsInPlaceMut<'a> {
    pub fn new(s: &'a mut str) -> Self {
        CharsInPlaceMut(s)
    }
}

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

impl<'a> DoubleEndedIterator for CharsInPlaceMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_char_len = self.0.chars().rev().next()?.len_utf8();

        let this: &mut str = core::mem::take(&mut self.0);
        let (rest, this) = this.split_at_mut(this.len() - next_char_len);
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
