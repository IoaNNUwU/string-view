 ## String View

 #### Work with views into string slices. Safely extend, reduce without losing parent string size.

 #### Use in-place modifications to speed up your code.

 Example:

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
 view.shrink_to_end();
 view.extend_while(|_| true);
 view.reduce_while(|ch| ch == ' ' || ch == '\n' || ch == '}');
 assert_eq!(view.as_str(), r#"let text = "Hello World";"#);
 
 view.reduce_while(|ch| ch == ';');
 view.reduce(1);
 view.shrink_to_end();
 view.extend_left_while(|ch| ch != '"');
 assert_eq!(view.as_str(), "Hello World");
 ```