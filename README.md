# Word Path game

Here is a game:

Having all dictionary words of length 5, choose a start word and an end word, for example: `phone` and `board`. Then try to find a path between these two words. You can change a letter of the word each step, and every intermediate word has to be in the dictionary. A possible path between `phone` and `board` is `phone, phons, poons, boons, boors, boars, board`. Simple to understand, but tricky to play.

This repo is an implementation in Rust of a solver that computes the best path between two words given a dictionary of equal-length words. This implementation is designed to be as efficient as possible, finding the best path between `board` and `phone` takes on average 24 ms on my machine.

## Usage

Compile with:
```
cargo build --release
```

Generate a dictionary of words of length 5 (or any length you want):
```
target/release/wordpathgame extract-words --len 5 words.txt words5.txt
```

Find the best path between two words:
```
target/release/wordpathgame find-path words5.txt board phone
```

## Benchmarks

Run the [Criterion](https://crates.io/crates/criterion) benchmarks (dictionary parsing, path search, and both) on the 5-letter words of `words.txt`:
```
cargo bench
```

Reports are written to `target/criterion/report/index.html`.

## Notes

This implementation was designed with an english dictionary in mind. The dictionary given to `find-path` must follow these rules:

- one word per line, with LF or CRLF line endings (empty lines are ignored);
- only lowercase letters `a` to `z`: no uppercase, accents, digits, apostrophes or hyphens;
- all words have the same length, at most 12 letters (each word is packed into a 64-bit integer, 5 bits per letter).

`find-path` stops with an error naming the first word that breaks a rule. The start and end words must follow the same rules, otherwise they are reported as not in the dictionary.
