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
fn trim_std(b: Bencher) {
    let input = String::from(HELLO);
    b.bench_local(|| {
        black_box(input.trim_matches(char::is_whitespace));
    });
}

// str::trim_mut should be as fast as str::trim

#[divan::bench(sample_count = 10_000)]
fn trim_mut_string_view(b: Bencher) {
    let mut input = String::from(HELLO);
    b.bench_local(|| {
        black_box(input.trim_matches_mut(char::is_whitespace));
    });
}
