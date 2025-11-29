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

## Notes

This implementation was designed with an english dictionary in mind, so only ASCII words are supported.
