use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use wordpathgame::{Dictionary, PathResult};

/// The 5-letter words of `words.txt`, as `extract-words --len 5` would produce.
fn five_letter_words() -> Vec<u8> {
    let words = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/words.txt"))
        .expect("words.txt should be at the crate root");

    let mut out = Vec::new();
    for word in words
        .split(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .filter(|word| word.len() == 5)
    {
        out.extend_from_slice(word);
        out.push(b'\n');
    }
    out
}

fn bench(c: &mut Criterion) {
    let data = five_letter_words();
    let dict = Dictionary::parse(&data).unwrap();

    assert!(matches!(
        dict.find_path("board", "phone"),
        PathResult::Found(_)
    ));

    c.bench_function("parse", |b| {
        b.iter(|| Dictionary::parse(black_box(&data)).unwrap())
    });

    c.bench_function("search board-phone", |b| {
        b.iter(|| dict.find_path(black_box("board"), black_box("phone")))
    });

    c.bench_function("parse + search board-phone", |b| {
        b.iter(|| {
            Dictionary::parse(black_box(&data))
                .unwrap()
                .find_path(black_box("board"), black_box("phone"))
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
