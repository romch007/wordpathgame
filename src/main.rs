use anyhow::anyhow;
use memmap::MmapOptions;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
enum Command {
    /// Extract words of certain length from a list of words
    ExtractWords {
        /// Length of the resulting words
        #[arg(long)]
        len: usize,

        /// Original list of words
        words: PathBuf,

        /// Output path
        extracted_words: PathBuf,
    },
    /// Find a path across two words
    FindPath {
        words: PathBuf,
        start_word: String,
        end_word: String,
    },
}

const ALPHA: &[u8] = "abcdefghijklmnopqrstuvwxyz".as_bytes();

fn extract_words(words: &Path, extracted_words: &Path, len: usize) -> anyhow::Result<()> {
    let words = File::open(words)?;
    let reader = BufReader::new(words);

    let extracted_words = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(extracted_words)?;

    let mut writer = BufWriter::new(extracted_words);

    for line in reader.lines() {
        let line = line?;

        if line.len() == len {
            writer.write_all(line.as_bytes())?;
            writer.write_all(&[b'\n'])?;
        }
    }

    Ok(())
}

type Word<'a> = &'a [u8];
type WordList<'a> = HashSet<Word<'a>>;
type Dictionnary<'a> = HashMap<Word<'a>, WordList<'a>>;

fn find_path(words: &Path, start_word: &str, end_word: &str) -> anyhow::Result<()> {
    // read the words
    let words = File::open(words)?;
    let words = unsafe { MmapOptions::new().map(&words)? };
    let words = words.split(|&b| b == b'\n').collect::<WordList>();

    println!("{} words were loaded", words.len());

    // generate the dictionnary
    let mut dict = Dictionnary::new();

    for word in &words {
        compute_neighbors(word, &words, &mut dict)?;
    }

    std::mem::drop(words);

    // find the path
    let start_word = start_word.as_bytes();
    let end_word = end_word.as_bytes();

    for word in [start_word, end_word] {
        if !dict.contains_key(word) {
            println!(
                "'{}' is not in the dictionnary",
                std::str::from_utf8(start_word)?
            );

            return Ok(());
        }
    }

    let mut path = VecDeque::with_capacity(1);
    path.push_back(start_word);

    let mut used = HashMap::new();
    let mut previous = HashMap::new();
    used.insert(start_word, true);
    previous.insert(start_word, &[] as Word);

    while !path.is_empty() {
        let current_word = path.pop_front().ok_or(anyhow!("path was empty???"))?;

        let neighbors = dict
            .get(current_word)
            .ok_or(anyhow!("value not in dict???"))?;

        for neighbor in neighbors {
            if !*used.get(neighbor).unwrap_or(&false) {
                used.insert(neighbor, true);
                path.push_back(neighbor);
                previous.insert(neighbor, current_word);
            }
        }
    }

    if !used[end_word] {
        println!("no path found");
    } else {
        let mut value = end_word;
        let mut reverse_path = Vec::new();
        while !value.is_empty() {
            reverse_path.push(value);
            value = previous[value];
        }

        println!("found path:");
        for part in reverse_path.into_iter().rev() {
            let part_str = std::str::from_utf8(part)?;
            println!("  - {part_str}");
        }
    }

    Ok(())
}

fn compute_neighbors<'a>(
    word: Word<'a>,
    available_words: &WordList<'a>,
    dict: &mut Dictionnary<'a>,
) -> anyhow::Result<()> {
    let mut neighbors = WordList::new();
    let mut owned_word = word.to_owned();

    for idx in 0..owned_word.len() {
        for &letter in ALPHA {
            let original_letter = owned_word[idx];

            if original_letter == letter {
                continue;
            }

            owned_word[idx] = letter;

            if let Some(neighbor) = available_words.get(owned_word.as_slice()) {
                neighbors.insert(*neighbor);
            };

            owned_word[idx] = original_letter;
        }
    }

    dict.insert(word, neighbors);

    Ok(())
}

fn main() {
    let command = Command::parse();

    match command {
        Command::ExtractWords {
            len,
            words,
            extracted_words,
        } => extract_words(&words, &extracted_words, len).unwrap(),
        Command::FindPath {
            words,
            start_word,
            end_word,
        } => find_path(&words, &start_word, &end_word).unwrap(),
    };
}