use core::error::Error;
use core::fmt::{Debug, Display};

/// Immutable view into string slice.
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
///
/// See [`StringViewMut`] for mutable version.
pub struct StringView<'a>(View<&'a str>);

impl<'a> StringView<'a> {
    /// Creates [`StringView`] of a whole string slice.
    ///
    /// See [`StringView::new_part`] to view part of a string slice.
    pub fn new(base: &'a str) -> Self {
        Self(View {
            base,
            view_start: 0,
            view_len: base.len(),
        })
    }

    /// Creates [`StringView`] of a part of string slice using 2 byte indices.
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
    /// Or using [`StrExt`](super::StrExt) extension trait:
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
        Self(View {
            base,
            view_start,
            view_len: view_end - view_start,
        })
    }

    /// Byte index of view start inside base string slice.
    pub fn start(&self) -> usize {
        self.0.start()
    }

    /// Byte index of view end inside base string slice.
    pub fn end(&self) -> usize {
        self.0.end()
    }

    pub fn as_str(&self) -> &'a str {
        &self.0.base[self.0.view_start..self.0.view_start + self.0.view_len]
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
        self.0.shrink_to_right();
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
        self.0.shrink_to_left();
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
        self.0.extend_right(n);
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
        self.0.try_extend_right(n)
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
    pub fn extend_right_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.extend_right_while(func);
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
        self.0.reduce_right(n);
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
        self.0.try_reduce_right(n)
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
    pub fn reduce_right_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.reduce_right_while(func);
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
        self.0.extend_left(n);
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
        self.0.try_extend_left(n)
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
    pub fn extend_left_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.extend_left_while(func);
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
        self.0.reduce_left(n);
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
        self.0.try_reduce_left(n)
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
    pub fn reduce_left_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.reduce_left_while(func);
    }

    /// Reduces string view from left and right while `func` returns `true`
    ///
    /// ```
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("  \n   Hello  \n \t  ");
    /// let mut view = text.view();
    ///
    /// view.trim_while(char::is_whitespace);
    ///
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn trim_while<F: FnMut(char) -> bool>(&mut self, func: F) {
        self.0.trim_while(func);
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

/// Mutable view into string slice.
///
/// Holds parent `str` info which allows to safely extend this view with parent
/// size in mind.
///
/// ```rust
/// use string_view::StrExt;
///
/// let mut text = String::from("Hello World");
/// let mut view = text.view_part_mut(6, 11);
/// assert_eq!(view.as_str(), "World");
///
/// view.extend_left(6);
/// assert_eq!(view.as_str(), "Hello World");
///
/// view.reduce_right_while(char::is_alphabetic);
/// assert_eq!(view.as_str(), "Hello ");
/// ```
///
/// ### Convert to `&mut str` while in use:
///
/// ```rust
/// use string_view::StrExt;
///
/// let mut text = String::from("Hello World");
/// let mut view = text.view_part_mut(6, 11);
/// assert_eq!(view.as_str(), "World");
///
/// view.as_str_mut().chars_in_place_mut().nth(3).unwrap().replace('L');
/// assert_eq!(view.as_str(), "WorLd");
/// assert_eq!(text, "Hello WorLd");
///
/// ```
pub struct StringViewMut<'a>(View<&'a mut str>);

impl<'a> StringViewMut<'a> {
    /// Creates [`StringViewMut`] of a whole string slice.
    ///
    /// see [`StringViewMut::new_part`] to view part of a string slice.
    pub fn new(base: &'a mut str) -> Self {
        Self(View {
            view_len: base.len(),
            base,
            view_start: 0,
        })
    }

    /// Creates [`StringViewMut`] of a part of string slice using 2 byte indices.
    ///
    /// ```rust
    /// use string_view::StringViewMut;
    ///
    /// let mut text = String::from("Hello World");
    /// let mut view = StringViewMut::new_part(&mut text, 6, 11);
    ///
    /// assert_eq!(view.as_str(), "World");
    /// ```
    ///
    /// Or using [`StrExt`](super::StrExt) extension trait:
    ///
    /// ```rust
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("Hello World");
    /// let mut view = text.view_part_mut(6, 11);
    ///
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn new_part(base: &'a mut str, view_start: usize, view_end: usize) -> Self {
        assert!(
            view_end >= view_start,
            "View end index cannot be less than start index"
        );
        Self(View {
            base,
            view_start,
            view_len: view_end - view_start,
        })
    }

    /// Byte index of view start inside base String.
    pub fn start(&self) -> usize {
        self.0.start()
    }

    /// Byte index of view end inside base String.
    pub fn end(&self) -> usize {
        self.0.end()
    }

    pub fn as_str(&self) -> &str {
        &self.0.base[self.0.view_start..self.0.view_start + self.0.view_len]
    }

    pub fn as_str_mut(&mut self) -> &mut str {
        &mut self.0.base[self.0.view_start..self.0.view_start + self.0.view_len]
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
    /// let mut text = String::from("Hello World");
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.shrink_to_right();
    /// view.extend_left_while(char::is_alphabetic);
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn shrink_to_right(&mut self) {
        self.0.shrink_to_right();
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
    /// let mut text = String::from("Hello World");
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.shrink_to_left();
    /// view.extend_right_while(char::is_alphabetic);
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn shrink_to_left(&mut self) {
        self.0.shrink_to_left();
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
    /// let mut text = String::from("Hello World");
    /// let mut view = text.view_part_mut(0, 5);
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend_right(6);
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_right(&mut self, n: usize) {
        self.0.extend_right(n);
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
    /// let mut text = String::from("Hello World");
    ///
    /// let mut view = text.view_part_mut(0, 5);
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
        self.0.try_extend_right(n)
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
    /// let mut text = String::from("Hello World !!!");
    ///
    /// let mut view = text.view_part_mut(0, 2);
    /// assert_eq!(view.as_str(), "He");
    ///
    /// view.extend_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello");
    ///
    /// view.extend_right(1);
    /// view.extend_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_right_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.extend_right_while(func);
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
    /// let mut text = String::from("Hello World");
    ///
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.reduce_right(6);
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn reduce_right(&mut self, n: usize) {
        self.0.reduce_right(n);
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
    /// let mut text = String::from("One and only Hello World");
    ///
    /// let mut view = text.view_part_mut(13, 24);
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
        self.0.try_reduce_right(n)
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
    /// let mut text = String::from("Hello World !!!");
    ///
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World !!!");
    ///
    /// view.reduce_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello World ");
    ///
    /// view.reduce_right(1);
    /// view.reduce_right_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "Hello ");
    /// ```
    pub fn reduce_right_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.reduce_right_while(func);
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
    /// let mut text = String::from("Hello World");
    /// let mut view = text.view_part_mut(6, 11);
    /// assert_eq!(view.as_str(), "World");
    ///
    /// view.extend_left(6);
    /// assert_eq!(view.as_str(), "Hello World");
    /// ```
    pub fn extend_left(&mut self, n: usize) {
        self.0.extend_left(n);
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
    /// let mut text = String::from("Hello World");
    ///
    /// let mut view = text.view_part_mut(6, 11);
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
        self.0.try_extend_left(n)
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
    /// let mut text = String::from("Hello World !!!");
    ///
    /// let mut view = text.view_part_mut(14, 15);
    /// assert_eq!(view.as_str(), "!");
    ///
    /// view.extend_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "!!!");
    ///
    /// view.extend_left(1);
    /// view.extend_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), "World !!!");
    /// ```
    pub fn extend_left_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.extend_left_while(func);
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
    /// let mut text = String::from("Hello World");
    ///
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World");
    ///
    /// view.reduce_left(6);
    /// assert_eq!(view.as_str(), "World");
    /// ```
    pub fn reduce_left(&mut self, n: usize) {
        self.0.reduce_left(n);
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
    /// let mut text = String::from("One and only Hello World");
    ///
    /// let mut view = text.view_part_mut(13, 24);
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
        self.0.try_reduce_left(n)
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
    /// let mut text = String::from("Hello World !!!");
    ///
    /// let mut view = text.view_mut();
    /// assert_eq!(view.as_str(), "Hello World !!!");
    ///
    /// view.reduce_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), " World !!!");
    ///
    /// view.reduce_left(1);
    /// view.reduce_left_while(|ch| ch != ' ');
    /// assert_eq!(view.as_str(), " !!!");
    /// ```
    pub fn reduce_left_while<F>(&mut self, func: F)
    where
        F: FnMut(char) -> bool,
    {
        self.0.reduce_left_while(func);
    }

    /// Reduces string view from left and right while `func` returns `true`
    ///
    /// ```
    /// use string_view::StrExt;
    ///
    /// let mut text = String::from("  \n   Hello  \n \t  ");
    /// let mut view = text.view_mut();
    ///
    /// view.trim_while(char::is_whitespace);
    ///
    /// assert_eq!(view.as_str(), "Hello");
    /// ```
    pub fn trim_while<F: FnMut(char) -> bool>(&mut self, func: F) {
        self.0.trim_while(func);
    }
}

impl Debug for StringViewMut<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StringViewMut<'_> {
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

struct View<T: AsRef<str>> {
    base: T,
    view_start: usize,
    view_len: usize,
}

impl<T: AsRef<str>> View<T> {
    pub fn start(&self) -> usize {
        self.view_start
    }
    pub fn end(&self) -> usize {
        self.view_start + self.view_len
    }

    pub fn shrink_to_right(&mut self) {
        self.view_start += self.view_len;
        self.view_len = 0;
    }

    pub fn shrink_to_left(&mut self) {
        self.view_len = 0;
    }

    pub fn extend_right(&mut self, n: usize) {
        self.try_extend_right(n)
            .expect("Unable to extend string view to the right")
    }

    pub fn try_extend_right(&mut self, n: usize) -> Result<(), BaseStringIsTooShort<RIGHT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base.as_ref()[self.end()..].chars();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(BaseStringIsTooShort)?.len_utf8();
        }
        self.view_len += combined_len;
        Ok(())
    }

    pub fn extend_right_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;

        for ch in self.base.as_ref()[self.end()..].chars() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_len += combined_len;
    }

    pub fn reduce_right(&mut self, n: usize) {
        self.try_reduce_right(n)
            .expect("Unable to reduce string view from the right")
    }

    pub fn try_reduce_right(&mut self, n: usize) -> Result<(), ViewIsTooShort<RIGHT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base.as_ref()[self.start()..self.end()].chars().rev();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(ViewIsTooShort)?.len_utf8();
        }
        self.view_len -= combined_len;
        Ok(())
    }

    pub fn reduce_right_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;
        for ch in self.base.as_ref()[self.start()..self.end()].chars().rev() {
            if func(ch) {
                combined_len += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.view_len -= combined_len;
    }

    pub fn extend_left(&mut self, n: usize) {
        self.try_extend_left(n)
            .expect("Unable to extend string view to the left")
    }

    pub fn try_extend_left(&mut self, n: usize) -> Result<(), BaseStringIsTooShort<LEFT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base.as_ref()[..self.start()].chars().rev();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(BaseStringIsTooShort)?.len_utf8();
        }
        self.view_start -= combined_len;
        self.view_len += combined_len;
        Ok(())
    }

    pub fn extend_left_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;

        for ch in self.base.as_ref()[..self.start()].chars().rev() {
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

    pub fn reduce_left(&mut self, n: usize) {
        self.try_reduce_left(n)
            .expect("Unable to reduce string view from the left")
    }

    pub fn try_reduce_left(&mut self, n: usize) -> Result<(), ViewIsTooShort<LEFT>> {
        let mut combined_len = 0;
        let mut char_iter = self.base.as_ref()[self.start()..self.end()].chars();
        for _ in 0..n {
            combined_len += char_iter.next().ok_or(ViewIsTooShort)?.len_utf8();
        }
        self.view_start += combined_len;
        self.view_len -= combined_len;
        Ok(())
    }

    pub fn reduce_left_while<F>(&mut self, mut func: F)
    where
        F: FnMut(char) -> bool,
    {
        let mut combined_len = 0;
        for ch in self.base.as_ref()[self.start()..self.end()].chars() {
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

    pub fn trim_while<F: FnMut(char) -> bool>(&mut self, mut func: F) {
        self.reduce_left_while(&mut func);
        self.reduce_right_while(&mut func);
    }
}
