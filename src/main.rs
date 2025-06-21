use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

use std::{collections::HashMap, fs};

use chrono::{DateTime, Days, TimeZone, Utc};
use edge_map::EdgeMap;
use game::{Grid, Level};
use generator::LevelGenerator;
use position::Direction;
use solver::WordList;

mod edge_map;
mod game;
mod generator;
mod position;
mod solver;

type DistanceMap = EdgeMap<usize>;
type TurnsMap = EdgeMap<(usize, Option<Direction>)>;

const START_DATE: &str = "2025-05-03 12:12:12Z";
const LEVEL_COUNT: usize = 365;
const WORDS_PATH: &str = "assets/easy_words.json";
const OUTPUT_FOLDER: &str = "assets/output";

/// Generates a supplied amount of levels that satisfy the predicate function.
fn generate_levels(
    word_list: WordList,
    amount: usize,
    size: (usize, usize),
    pred: fn(&Level) -> bool,
) -> Vec<Level> {
    let mut result: Vec<Level> = Vec::new();
    let (rows, cols) = size;
    while result.len() < amount {
        let generator = LevelGenerator::from_grid(Grid::new(rows, cols));
        if let Some(level) = generator.attempt_generate_level(&word_list, 20) {
            if pred(&level) {
                println!("Added level: {}", result.len());
                result.push(level);
            }
        }
    }

    result
}

/// A filter that returns true if the level's solution has the supplied
/// minimum average letter count.
fn has_minimum_avg_letter_count<const SIZE: usize>(level: &Level) -> bool {
    // Avg letter count must be greater than 3
    let letter_count = level.words.iter().fold(0, |count, word| count + word.len());
    let avg_count = letter_count / level.words.len();
    return avg_count >= SIZE;
}

/// Add available letters to this level to make it easier, and give more
/// potential solutions to the user. This is done by sampling the suppplied
/// letter frequencies.
fn increase_letters(level: &mut Level, frequencies: &HashMap<char, usize>) {
    let letter_count = level
        .words
        .iter()
        .fold(0, |count, word| count + word.len() - 2);

    let freqs: Vec<_> = frequencies.iter().collect();
    let choices: Vec<char> = freqs.iter().map(|i| i.0).copied().collect();
    let weights: Vec<usize> = freqs.iter().map(|i| i.1).copied().collect();
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = rand::rng();

    let mut padded_word = String::new();
    while padded_word.len() < letter_count / 2 {
        padded_word.push(choices[dist.sample(&mut rng)])
    }
    level.words.push(padded_word);
}

/// Return the name of the level in YYYY-MM-DD format.
fn level_name(start_date: &DateTime<Utc>, index: u64) -> String {
    let Some(date) = start_date.checked_add_days(Days::new(index)) else {
        return format!("{}", index);
    };
    format!("{}", date.format("%Y-%m-%d"))
}

fn main() {
    let word_list = WordList::from_path(WORDS_PATH).expect("Could not load words");
    let frequencies = word_list.frequencies();

    // Create the levels
    let mut levels = generate_levels(
        word_list,
        LEVEL_COUNT,
        (8, 8),
        has_minimum_avg_letter_count::<4>,
    );
    levels
        .iter_mut()
        .for_each(|level| increase_letters(level, &frequencies));

    levels.iter().for_each(Level::visualise);

    // Build their names and save them to disk
    let start_date = START_DATE.parse::<DateTime<Utc>>().unwrap();

    levels.iter().enumerate().for_each(|(i, level)| {
        let raw = serde_json::to_string(level).expect("Couldn't convert level");
        let name = level_name(&start_date, i as u64);
        let path = format!("{}/{}.json", OUTPUT_FOLDER, name);
        println!("{}", path);

        fs::write(path, raw).expect("Couldn't write.");
    });
}
