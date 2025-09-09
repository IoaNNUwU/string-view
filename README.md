## String View

#### Use in-place modifications to avoid allocations.

```rust
use string_view::StrExt;

let mut text: &mut str = &mut String::from("Hello World");

text.chars_in_place_mut()
    .filter(|ch| !ch.char().is_whitespace())
    .for_each(|mut ch| { ch.make_uppercase(); });

assert_eq!(text, "HELLO WORLD");
```

#### Work with views into string slices. Safely extend, reduce without losing parent string size.

```rust
let program_text = r#"
fn main() {
    let text = "Hello World";
}
"#;

use string_view::StrExt;

let mut view = program_text.view_part(0, 0);
view.extend_right_while(|ch| ch == ' ' || ch == '\n');
view.extend_right_while(char::is_alphabetic);
view.reduce_left_while(|ch| ch == ' ' || ch == '\n');
assert_eq!(view.as_str(), "fn");

view.try_extend_right(1).unwrap();
view.extend_right_while(char::is_alphabetic);
view.try_extend_right(2).unwrap();
assert_eq!(view.as_str(), "fn main()");

view.extend_right_while(|ch| ch == ' ' || ch == '\n' || ch == '{');
view.shrink_to_right();
view.extend_right_while(|_| true);
view.reduce_right_while(|ch| ch == ' ' || ch == '\n' || ch == '}');
assert_eq!(view.as_str(), r#"let text = "Hello World";"#);

view.reduce_right_while(|ch| ch == ';');
view.reduce_right(1);
view.shrink_to_right();
view.extend_left_while(|ch| ch != '"');
assert_eq!(view.as_str(), "Hello World");
```
