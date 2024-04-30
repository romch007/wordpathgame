use anyhow::anyhow;
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
    /// Generate a dictionnary from a list of words of equal length
    GenerateDict {
        /// List of words
        words: PathBuf,
        /// Output path
        dict: PathBuf,
    },
    /// Find a path across two words from a dictionnary
    FindPath {
        dict: PathBuf,
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

fn generate_dict(words: &Path, dict: &Path) -> anyhow::Result<()> {
    let available_words = File::open(words)?;
    let mut reader = BufReader::new(available_words);

    let mut available_words = HashSet::new();
    let mut buf = vec![];

    loop {
        let nread = reader.read_until(b'\n', &mut buf)?;

        if nread == 0 {
            break;
        }

        let word = &buf[0..nread - 1];

        available_words.insert(word.to_owned());
        buf.clear();
    }

    println!("{} words were read", available_words.len());

    let dict = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(dict)?;

    let mut writer = BufWriter::new(dict);

    for word in &available_words {
        compute_neighbors(word, &available_words, &mut writer)?;
    }

    Ok(())
}

fn compute_neighbors(
    word: &[u8],
    available_words: &HashSet<Vec<u8>>,
    mut dict: impl Write,
) -> anyhow::Result<()> {
    dict.write_all(word)?;

    let mut word = word.to_owned();

    for idx in 0..word.len() {
        for &letter in ALPHA {
            let original_letter = word[idx];

            if original_letter == letter {
                continue;
            }

            word[idx] = letter;

            if available_words.contains(&word) {
                dict.write_all(&[b' '])?;
                dict.write_all(&word)?;
            }

            word[idx] = original_letter;
        }
    }

    dict.write_all(&[b'\n'])?;

    Ok(())
}

fn find_path(dict: &Path, start_word: &str, end_word: &str) -> anyhow::Result<()> {
    let dict = File::open(dict)?;
    let mut reader = BufReader::new(dict);

    // load the dictionary
    let mut dict = HashMap::new();
    let mut buf = vec![];

    loop {
        let nread = reader.read_until(b'\n', &mut buf)?;

        if nread == 0 {
            break;
        }

        let line = &buf[0..nread - 1];
        let mut parts = line.split(|&b| b == b' ');

        let word = parts.next().ok_or(anyhow!("empty line???"))?.to_owned();
        let neighbors = parts.map(|word| word.to_owned()).collect::<HashSet<_>>();

        dict.insert(word, neighbors);
        buf.clear();
    }

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
    previous.insert(start_word, &[] as &[u8]);

    while !path.is_empty() {
        let current_word = path.pop_front().ok_or(anyhow!("path was empty???"))?;

        let neighbors = dict
            .get(current_word)
            .ok_or(anyhow!("value not in dict???"))?;

        for neighbor in neighbors {
            let neighbor = neighbor.as_slice();
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

fn main() {
    let command = Command::parse();

    match command {
        Command::ExtractWords {
            len,
            words,
            extracted_words,
        } => extract_words(&words, &extracted_words, len).unwrap(),
        Command::GenerateDict { words, dict } => generate_dict(&words, &dict).unwrap(),
        Command::FindPath {
            dict,
            start_word,
            end_word,
        } => find_path(&dict, &start_word, &end_word).unwrap(),
    };
}
