## String View

```toml
[dependencies]
string-view = "0.3"
```

#### Use in-place modifications to avoid allocations.

```rust
# extern crate std;
# use std::string::String;
use string_view::StrExt;

let text: &mut str = &mut String::from("Hello World");

text.chars_in_place()
    .filter(|ch| !ch.char().is_whitespace())
    .for_each(|ch| ch.make_uppercase());

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
view.extend_while(|ch| ch == ' ' || ch == '\n');
view.extend_while(char::is_alphabetic);
view.reduce_left_while(|ch| ch == ' ' || ch == '\n');
assert_eq!(view.as_str(), "fn");

view.try_extend(1).unwrap();
view.extend_while(char::is_alphabetic);
view.try_extend(2).unwrap();
assert_eq!(view.as_str(), "fn main()");

view.extend_while(|ch| ch == ' ' || ch == '\n' || ch == '{');
view.shrink_left();
view.extend_while(|_| true);
view.reduce_while(|ch| ch == ' ' || ch == '\n' || ch == '}');
assert_eq!(view.as_str(), r#"let text = "Hello World";"#);

view.reduce_while(|ch| ch == ';');
view.reduce(1);
view.shrink_left();
view.extend_left_while(|ch| ch != '"');
assert_eq!(view.as_str(), "Hello World");
```