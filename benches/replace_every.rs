use std::hint::black_box;

use divan::Bencher;
use string_view::StrExt;

fn main() {
    divan::main();
}

// Replace every character with the next one

const ALP: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";

#[divan::bench(sample_count = 10_000)]
fn alphabet_std_push(b: Bencher) {
    let mut input = String::from(ALP);

    b.bench_local(|| {
        
        let mut out = String::with_capacity(ALP.len());

        for (idx, _) in input.char_indices() {
            let next = &ALP.get(idx + 1..idx + 2).unwrap_or("a");
            out.push_str(&next);
        }

        input = out;

        black_box(&mut input);
    });
}

#[divan::bench(sample_count = 10_000)]
fn alphabet_string_view_in_place(b: Bencher) {
    let mut input = String::from(ALP);

    b.bench_local(|| {

        for (idx, mut ch) in input.chars_in_place_mut().enumerate() {
            let next = &ALP.get(idx + 1..idx + 2).unwrap_or("a");
            ch.replace('*').unwrap();
        }

        black_box(&mut input);
    });
}