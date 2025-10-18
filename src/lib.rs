#![doc = include_str!("../README.md")]
#![no_std]

mod string_view;
pub use string_view::*;

mod char;
pub use crate::char::*;

#[cfg(test)]
mod test;

pub trait StrExt {
    /// Returns [`StringView`] of a whole string slice.
    fn view(&self) -> StringView<'_>;

    /// Returns [`StringViewMut`] of a whole string slice.
    fn view_mut(&mut self) -> StringViewMut<'_>;

    /// Returns [`StringView`] of a part of a string slice.
    fn view_part(&self, start_idx: usize, end_idx: usize) -> StringView<'_>;

    /// Returns [`StringViewMut`] of a part of a string slice.
    fn view_part_mut(&mut self, start_idx: usize, end_idx: usize) -> StringViewMut<'_>;

    /// Start and end byte indices of this [`Char`] inside base [`str`].
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

    /// Returns in-place characters interator of this string slice
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text = "Hello";
    ///
    /// let mut chars = text.chars_in_place();
    ///
    /// assert_eq!(chars.next().unwrap(), "H");
    /// assert_eq!(chars.next().unwrap(), "e");
    /// assert_eq!(chars.next().unwrap(), 'l');
    /// assert_eq!(chars.next().unwrap(), 'l');
    /// assert_eq!(chars.next().unwrap(), 'o');
    /// ```
    fn chars_in_place(&self) -> CharsInPlace<'_>;

    /// Returns mutable in-place characters interator of this string slice
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("Hello");
    ///
    /// for mut ch in text.chars_in_place_mut() {
    ///     ch.make_uppercase();
    /// }
    ///
    /// assert_eq!(text, "HELLO");
    /// ```
    fn chars_in_place_mut(&mut self) -> CharsInPlaceMut<'_>;

    /// Makes [`str`] characters lowercase in-place where appropriate.
    ///
    /// Doesn't change character if lowercase variant takes different amount of bytes.
    ///
    /// Size checks happen at runtime.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("HELLO World");
    /// text.make_lowercase();
    /// assert_eq!(text, "hello world");
    /// ```
    /// ### Unicode
    /// ```
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("ПРИВЕТ Мир");
    /// text.make_lowercase();
    /// assert_eq!(text, "привет мир");
    /// ```
    fn make_lowercase(&mut self);

    /// Makes string slice characters uppercase in-place where appropriate.
    ///
    /// Doesn't change character if uppercase variant takes different amount of bytes.
    ///
    /// Size checks happen at runtime.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("Hello World");
    /// text.make_uppercase();
    /// assert_eq!(text, "HELLO WORLD");
    /// ```
    /// ### Unicode
    /// ```
    /// use string_view::StrExt;
    ///
    /// let text: &mut str = &mut String::from("Привет мир");
    /// text.make_uppercase();
    /// assert_eq!(text, "ПРИВЕТ МИР");
    /// ```
    fn make_uppercase(&mut self);

    /// Replaces whole string slice with another one with same length in-place. Useful if
    /// this `&mut str` is part of another `&mut str`.
    ///
    /// **Panics** if replacement string slice has different length.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("Hello World");
    ///
    /// (&mut text[6..11]).replace_in_place("WORLD");
    /// assert_eq!(text, "Hello WORLD");
    /// ```
    /// ### Unicode
    /// ```
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("Привет Мир");
    ///
    /// // Note both original slice and replacement take 6 bytes
    /// (&mut text[13..19]).replace_in_place("Worlds");
    /// assert_eq!(text, "Привет Worlds");
    /// ```
    fn replace_in_place(&mut self, rep: &str);

    /// Replaces all characters in this string slice with provided `char` in-place.
    ///
    /// **Panics** if argument has incompatible [length in `UTF-8` encoding](char::len_utf8).
    ///
    /// - Panics if [`str::len`] % [`char::len_utf8`] != 0
    /// - Never panics if [`char::len_utf8`] == 1 so its safe to use this with ASCII characters.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from(">>World<<");
    /// let len = text.len();
    ///
    /// let mut text_slice = &mut text[2..len - 2];
    /// assert_eq!(text_slice, "World");
    ///
    /// text_slice.replace_with_char('*');
    ///
    /// assert_eq!(text, ">>*****<<");
    /// ```
    /// ### Unicode
    /// ```
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from(">>Мир<<");
    /// let len = text.len();
    ///
    /// let mut text_slice = &mut text[2..len - 2];
    /// assert_eq!(text_slice, "Мир");
    /// assert_eq!(text_slice.len(), 6); // Note each char takes 2 bytes
    ///
    /// text_slice.replace_with_char('*');
    ///
    /// assert_eq!(text, ">>******<<"); // Note 6 stars - 2 for each char
    /// ```
    fn replace_with_char(&mut self, ch: char);

    /// Returns a mutable string slice with all prefixes and suffixes that match a pattern repeatedly removed.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("--Hello--World----");
    ///
    /// let subslice: &mut str = text.trim_matches_mut(|ch| ch == '-');
    /// assert_eq!(subslice, "Hello--World");
    /// ```
    fn trim_matches_mut<P: FnMut(char) -> bool>(&mut self, pat: P) -> &mut str;

    /// Returns a string slice with leading and trailing whitespace removed.
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("  Hello  World    ");
    ///
    /// let subslice: &mut str = text.trim_mut();
    /// assert_eq!(subslice, "Hello  World");
    /// ```
    fn trim_mut(&mut self) -> &mut str;
}

impl StrExt for str {
    fn view(&self) -> StringView<'_> {
        StringView::new(self)
    }

    fn view_mut(&mut self) -> StringViewMut<'_> {
        StringViewMut::new(self)
    }

    fn view_part(&self, start_idx: usize, end_idx: usize) -> StringView<'_> {
        StringView::new_part(self, start_idx, end_idx)
    }

    fn view_part_mut(&mut self, start_idx: usize, end_idx: usize) -> StringViewMut<'_> {
        StringViewMut::new_part(self, start_idx, end_idx)
    }

    fn chars_in_place(&self) -> CharsInPlace<'_> {
        CharsInPlace::new(self)
    }

    fn chars_in_place_mut(&mut self) -> CharsInPlaceMut<'_> {
        CharsInPlaceMut::new(self)
    }

    fn char_idx(&self, ch: Char) -> (usize, usize) {
        let str_start = self.as_ptr() as usize;
        let str_end = str_start + self.len();
        let ch_start = ch.as_str().as_ptr() as usize;
        let ch_end = ch_start + ch.as_str().len();

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

    fn make_lowercase(&mut self) {
        self.chars_in_place_mut().for_each(|mut ch| {
            let _ = ch.make_lowercase();
        });
    }

    fn make_uppercase(&mut self) {
        self.chars_in_place_mut().for_each(|mut ch| {
            let _ = ch.make_uppercase();
        });
    }

    fn replace_in_place(&mut self, rep: &str) {
        assert_eq!(
            self.len(),
            rep.len(),
            "replacement string slice has to have the same size as the original. Consider creating mutable subslice with different length"
        );
        // SAFETY: rep is str so self is valid after copy_from_slice
        unsafe {
            self.as_bytes_mut().copy_from_slice(rep.as_bytes());
        }
    }

    fn replace_with_char(&mut self, ch: char) {
        let len = self.len();
        let replacement_char_len = ch.len_utf8();

        assert!(
            len % replacement_char_len == 0,
            "This string slice cannot be fully replaced by this character. Consider creating mutable subslice with different length"
        );

        // SAFETY: length is OK, chars are being encoded - bytes are valid after copying.
        unsafe {
            let bytes = self.as_bytes_mut();

            for idx in 0..(len / replacement_char_len) {
                ch.encode_utf8(&mut bytes[idx..idx + replacement_char_len]);
            }
        }
    }

    fn trim_matches_mut<P: FnMut(char) -> bool>(&mut self, pat: P) -> &mut str {
        let trimmed = self.trim_matches(pat);

        let len = trimmed.len();

        // Pattern is unstable - using hacks.
        // https://github.com/rust-lang/rust/issues/27721
        let start_idx =
            trimmed as *const str as *const u8 as usize - self as *mut str as *mut u8 as usize;

        // SAFETY: start & end indices returned by `str::trim_matches`
        unsafe { self.get_unchecked_mut(start_idx..start_idx + len) }
    }

    fn trim_mut(&mut self) -> &mut str {
        self.trim_matches_mut(char::is_whitespace)
    }
}
