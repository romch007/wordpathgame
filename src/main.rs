use clap::Parser;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use wordpathgame::{Dictionary, PathResult};

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
            writer.write_all(b"\n")?;
        }
    }

    Ok(())
}

fn find_path(words: &Path, start_word: &str, end_word: &str) -> anyhow::Result<()> {
    let dict = Dictionary::parse(&std::fs::read(words)?)?;

    println!("{} words were loaded", dict.len());

    match dict.find_path(start_word, end_word) {
        PathResult::Found(path) => {
            println!("found path:");
            for word in path {
                println!("  - {word}");
            }
        }
        PathResult::NoPath => println!("no path found"),
        PathResult::NotInDictionary(word) => println!("'{word}' is not in the dictionary"),
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
        Command::FindPath {
            words,
            start_word,
            end_word,
        } => find_path(&words, &start_word, &end_word).unwrap(),
    };
}
