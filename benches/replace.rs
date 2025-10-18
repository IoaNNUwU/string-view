use std::hint::black_box;

use divan::Bencher;
use string_view::StrExt;

fn main() {
    divan::main();
}

const HELLO: &str = "                                                                       
                                                                                            
                                                                                            
                                        Hello                                               
                                                                                            
                                                                                            
                                                                                            ";

#[divan::bench(sample_count = 10_000)]
fn replace_std(b: Bencher) {
    let input = String::from(HELLO);
    b.bench_local(|| {
        black_box(input.replace(char::is_alphabetic, "*"));
    });
}

#[divan::bench(sample_count = 10_000)]
fn replace_std_unsafe(b: Bencher) {
    let mut input = String::from(HELLO);
    b.bench_local(|| {
        let subslice = input.trim();
        let len = subslice.len();

        let start_idx = subslice as *const str as *const u8 as usize - input.as_mut_ptr() as usize;
        let end_idx = start_idx + len;

        let hello = &mut input[start_idx..end_idx];

        unsafe {
            for byte in hello.as_bytes_mut() {
                *byte = b'*';
            }
        }

        black_box(&mut input);
    });
}

#[divan::bench(sample_count = 10_000)]
fn replace_string_view_string_view(b: Bencher) {
    let mut input = String::from(HELLO);
    b.bench_local(|| {
        let mut view = black_box(input.view_mut());

        view.reduce_left_while(char::is_whitespace);
        view.reduce_right_while(char::is_whitespace);

        view.as_str_mut().replace_with_char('*');

        black_box(view);
    });
}

#[divan::bench(sample_count = 10_000)]
fn replace_string_view_trim(b: Bencher) {
    let mut input = String::from(HELLO);
    b.bench_local(|| {
        black_box(input.trim_mut().replace_with_char('*'));
    });
}

// Check how much time 1 memset takes
#[divan::bench(sample_count = 10_000)]
fn replace_unfair(b: Bencher) {
    let mut input = String::from(HELLO);
    let mut view = input.view_mut();

    view.trim_while(char::is_whitespace);

    b.bench_local(|| {
        view.as_str_mut().replace_in_place("*****");
    });
}
