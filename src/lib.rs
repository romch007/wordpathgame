use anyhow::{anyhow, bail};
use fnv::FnvHashMap;

/// Bits used to store one letter in a packed word (`a` = 1 .. `z` = 26).
const LETTER_BITS: usize = 5;
const LETTER_MASK: u64 = (1 << LETTER_BITS) - 1;
/// Longest word that fits in a `u64` key.
pub const MAX_WORD_LEN: usize = u64::BITS as usize / LETTER_BITS;

/// Packs a lowercase ASCII word into a `u64`, 5 bits per letter.
fn encode(word: &[u8]) -> Option<u64> {
    if word.len() > MAX_WORD_LEN {
        return None;
    }

    word.iter().try_fold(0u64, |key, &c| {
        c.is_ascii_lowercase()
            .then(|| (key << LETTER_BITS) | u64::from(c - b'a' + 1))
    })
}

fn decode(key: u64, len: usize) -> String {
    (0..len)
        .rev()
        .map(|idx| (((key >> (LETTER_BITS * idx)) & LETTER_MASK) as u8 + b'a' - 1) as char)
        .collect()
}

/// Outcome of a path search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResult {
    /// Shortest path, from the start word to the end word (both included).
    Found(Vec<String>),
    /// Both words are in the dictionary, but no path links them.
    NoPath,
    /// This word is not in the dictionary.
    NotInDictionary(String),
}

/// A list of words of equal length.
pub struct Dictionary {
    /// packed word for each word id
    keys: Vec<u64>,
    /// packed word -> word id
    ids: FnvHashMap<u64, u32>,
    word_len: usize,
}

impl Dictionary {
    /// Parses a newline-separated list of words (LF or CRLF).
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        let mut word_len = None;
        let mut keys = Vec::new();
        let mut ids = FnvHashMap::default();

        let lines = data
            .split(|&b| b == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line));

        for word in lines.filter(|word| !word.is_empty()) {
            let word_str = || String::from_utf8_lossy(word);

            match word_len {
                None => word_len = Some(word.len()),
                Some(len) if len != word.len() => bail!(
                    "dictionary contains words of different lengths, offending word: '{}'",
                    word_str()
                ),
                Some(_) => {}
            }

            let key = encode(word).ok_or_else(|| {
                anyhow!(
                    "dictionary words must be lowercase a-z and at most {MAX_WORD_LEN} letters long, offending word: '{}'",
                    word_str()
                )
            })?;

            ids.entry(key).or_insert_with(|| {
                keys.push(key);
                (keys.len() - 1) as u32
            });
        }

        let Some(word_len) = word_len else {
            bail!("no word in dictionary")
        };

        Ok(Self {
            keys,
            ids,
            word_len,
        })
    }

    /// Number of distinct words.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Always false: parsing rejects empty dictionaries.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn id(&self, word: &str) -> Option<u32> {
        if word.len() != self.word_len {
            return None;
        }

        encode(word.as_bytes()).and_then(|key| self.ids.get(&key).copied())
    }

    /// Finds a shortest path from `start_word` to `end_word`, changing one letter per step.
    pub fn find_path(&self, start_word: &str, end_word: &str) -> PathResult {
        let mut endpoints = [0u32; 2];
        for (endpoint, word) in endpoints.iter_mut().zip([start_word, end_word]) {
            match self.id(word) {
                Some(id) => *endpoint = id,
                None => return PathResult::NotInDictionary(word.to_owned()),
            }
        }
        let [start, end] = endpoints;

        // breadth-first search; neighbors are computed only for visited words
        const UNVISITED: u32 = u32::MAX;
        let mut previous = vec![UNVISITED; self.keys.len()];
        previous[start as usize] = start;

        let mut queue = Vec::with_capacity(self.keys.len());
        queue.push(start);
        let mut head = 0;

        'search: while start != end && head < queue.len() {
            let current = queue[head];
            head += 1;

            let key = self.keys[current as usize];

            for idx in 0..self.word_len {
                let shift = LETTER_BITS * idx;
                let original_letter = (key >> shift) & LETTER_MASK;
                let base = key & !(LETTER_MASK << shift);

                for letter in 1..=26 {
                    if letter == original_letter {
                        continue;
                    }

                    if let Some(&neighbor) = self.ids.get(&(base | (letter << shift))) {
                        if previous[neighbor as usize] == UNVISITED {
                            previous[neighbor as usize] = current;

                            // BFS: the first time we reach the end is the shortest path
                            if neighbor == end {
                                break 'search;
                            }

                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        if previous[end as usize] == UNVISITED {
            return PathResult::NoPath;
        }

        let mut path = vec![end];
        let mut current = end;
        while current != start {
            current = previous[current as usize];
            path.push(current);
        }

        PathResult::Found(
            path.into_iter()
                .rev()
                .map(|id| decode(self.keys[id as usize], self.word_len))
                .collect(),
        )
    }
}
