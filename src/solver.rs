use rand::seq::IteratorRandom;
use serde_json;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::position::Position;

// NOTE: Could make the dictionary a map of lengths to tries.

#[derive(Debug)]
pub enum LoadWordsError {
    FileError,
    ParseError,
}

pub struct WordList(HashMap<usize, HashSet<String>>);

impl WordList {
    pub fn from_words(words: Vec<String>) -> Self {
        WordList(words.iter().fold(HashMap::new(), |mut res, word| {
            if !res.contains_key(&word.len()) {
                res.insert(word.len(), HashSet::new());
            }
            let set = res
                .get_mut(&word.len())
                .expect("Expected new set to be in result");
            set.insert(word.to_lowercase());
            res
        }))
    }

    pub fn size(&self) -> usize {
        self.0.iter().fold(0, |r, (_, s)| r + s.len())
    }

    pub fn from_path(path: &str) -> Result<WordList, LoadWordsError> {
        fs::read_to_string(path)
            .map_err(|_| LoadWordsError::FileError)
            .and_then(|raw| {
                serde_json::from_str::<Vec<String>>(raw.as_str())
                    .map_err(|_| LoadWordsError::ParseError)
            })
            .map(WordList::from_words)
    }

    pub fn is_word_valid(&self, word: &String) -> bool {
        return self
            .0
            .get(&word.len())
            .map(|set| set.contains(word))
            .unwrap_or(false);
    }

    pub fn frequencies(&self) -> HashMap<char, usize> {
        self.0.values().fold(HashMap::new(), |mut res, set| {
            for word in set.into_iter() {
                for letter in word.chars() {
                    let current = res.get(&letter).unwrap_or(&0);
                    res.insert(letter, current + 1);
                }
            }
            res
        })
    }

    pub fn find_constrained_words(&self, constraints: Vec<WordConstraint>) -> HashSet<String> {
        let max_index = constraints
            .iter()
            .map(|c| match c {
                WordConstraint::Length(index) => index,
                WordConstraint::CharAt(index, _) => index,
            })
            .max()
            .unwrap_or(&0);
        let valid_sets: Vec<_> = self
            .0
            .iter()
            .filter_map(
                |(size, set)| {
                    if size >= max_index { Some(set) } else { None }
                },
            )
            .collect();

        let candidates: HashSet<String> = valid_sets
            .into_iter()
            .fold(HashSet::new(), |s, s2| s.union(s2).cloned().collect());

        candidates
            .iter()
            .filter(|word| constraints.iter().all(|c| c.satisfies(word)))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum WordConstraint {
    Length(usize),
    CharAt(usize, char),
}

impl WordConstraint {
    pub fn satisfies(&self, word: &String) -> bool {
        match self {
            WordConstraint::Length(size) => word.len() == *size,
            WordConstraint::CharAt(index, letter) => {
                word.len() > *index && word.chars().nth(*index).unwrap() == *letter
            }
        }
    }
}

pub struct SolutionWord {
    pub start: Position,
    pub end: Position,
    pub word: String,
}

impl SolutionWord {
    pub fn first_letter(&self) -> char {
        self.word.chars().next().unwrap()
    }

    pub fn last_letter(&self) -> char {
        self.word.chars().last().unwrap()
    }
}

pub struct Solution {
    words: Vec<SolutionWord>,
    segments: Vec<(Position, Position)>,
}

impl Solution {
    pub fn new(segments: Vec<(Position, Position)>) -> Self {
        Self {
            words: Vec::new(),
            segments,
        }
    }

    pub fn all_words(&self) -> Vec<&String> {
        self.words.iter().map(|w| &w.word).collect()
    }

    pub fn last_word(&self) -> Option<&SolutionWord> {
        self.words.last()
    }

    pub fn is_complete(&self) -> bool {
        self.words.len() == self.segments.len()
    }

    pub fn next_segment(&self) -> Option<(Position, Position)> {
        if self.is_complete() {
            None
        } else {
            Some(self.segments[self.words.len()])
        }
    }

    pub fn add_word(&mut self, word: &String) {
        if let Some((start, end)) = self.next_segment() {
            self.words.push(SolutionWord {
                start,
                end,
                word: word.clone(),
            })
        }
    }

    pub fn next_constraints(&self) -> Vec<WordConstraint> {
        let Some((next_start, next_stop)) = self.next_segment() else {
            return Vec::new();
        };
        let next_length = next_start.manhattan_distance(next_stop) + 1;
        let mut constraints = vec![WordConstraint::Length(next_length)];
        let Some(word) = self.last_word() else {
            return constraints;
        };

        let start_letter = word.first_letter();
        let last_letter = word.last_letter();
        let start = word.start;
        let end = word.end;

        if next_start == end {
            constraints.push(WordConstraint::CharAt(0, last_letter))
        } else if next_start == start {
            constraints.push(WordConstraint::CharAt(0, start_letter))
        } else if next_stop == start {
            constraints.push(WordConstraint::CharAt(next_length - 1, start_letter))
        } else {
            constraints.push(WordConstraint::CharAt(next_length - 1, last_letter))
        }
        constraints
    }

    pub fn attempt_solve(&mut self, word_list: &WordList, max_attempts: usize) -> Option<()> {
        let mut attempts = 0;
        'solving: while attempts < max_attempts {
            while !self.is_complete() {
                let constraints = self.next_constraints();
                let candidates = word_list.find_constrained_words(constraints.clone());
                // Choose a random solution from candidates
                match candidates.iter().choose(&mut rand::rng()) {
                    Some(word) => {
                        self.add_word(word);
                    }
                    // No words fit
                    None => {
                        attempts += 1;
                        continue 'solving;
                    }
                }
            }
            // We completed the solution
            return Some(());
        }
        None
    }
}
