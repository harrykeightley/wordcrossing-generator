use rand::prelude::*;
use std::collections::HashSet;

use crate::{
    DistanceMap, TurnsMap,
    game::{Grid, Level},
    position::Position,
    solver::{Solution, WordList},
};

pub struct LevelGenerator {
    pub grid: Grid,
    pub free_space: HashSet<Position>,
    pub turns_map: TurnsMap,
    pub distance_map: DistanceMap,
}

impl LevelGenerator {
    pub fn from_grid(mut grid: Grid) -> Self {
        let free_space = grid.initialise_walls();
        let turns_map = grid.generate_turns_map();
        let distance_map = grid.generate_distance_map();

        Self {
            grid,
            free_space,
            turns_map,
            distance_map,
        }
    }

    pub fn attempt_generate_level(
        &self,
        word_list: &WordList,
        solver_retries: usize,
    ) -> Option<Level> {
        let (start, goal) = self.choose_start_and_goal()?;

        let mut level = Level {
            start,
            goal,
            grid: self.grid.clone(),
            words: Vec::new(),
        };

        let junctions = self.find_path_junctions(start, goal);
        let segments = LevelGenerator::extract_segments(junctions);

        let mut solution = Solution::new(segments);
        if let None = solution.attempt_solve(word_list, solver_retries) {
            return None;
        }

        level.words = solution.all_words().into_iter().cloned().collect();
        Some(level)
    }

    pub fn choose_start_and_goal(&self) -> Option<(Position, Position)> {
        let start = self.free_space.iter().choose(&mut rand::rng())?;
        let start_deltas = self.distance_map.0.get(start)?;
        let start_turns = self.turns_map.0.get(start)?;

        let mut candidates = self.free_space.clone();
        candidates.remove(start);

        let mut candidates: Vec<_> = candidates.iter().collect();
        // Take into account distance and turns
        candidates.sort_by_key(|&p| {
            start_deltas.get(p).unwrap_or(&0) + start_turns.get(p).map(|v| v.0).unwrap_or(0)
        });

        // Choose from latter third
        let count = candidates.len();
        let candidates: Vec<_> = candidates.into_iter().skip(count * 2 / 3).collect();

        let start = *start;
        let goal = candidates.into_iter().choose(&mut rand::rng())?.clone();
        Some((start, goal))
    }

    fn find_path_junctions(&self, start: Position, goal: Position) -> Vec<Position> {
        let mut position = start;
        let mut path = vec![start];
        let mut turns_left = self.turns_map.get(position, goal).unwrap().0.clone();

        while position != goal {
            let (turns, direction) = self.turns_map.get(position, goal).unwrap();
            if turns_left != *turns {
                turns_left = *turns;
                path.push(position);
            }
            position = position.step_in_direction(direction.unwrap())
        }
        path.push(goal);
        path
    }

    fn extract_segments(junctions: Vec<Position>) -> Vec<(Position, Position)> {
        let mut segments: Vec<(Position, Position)> = Vec::new();
        for (i, junction) in junctions.iter().enumerate().skip(1) {
            segments.push((junctions[i - 1], *junction))
        }
        // Get in top-left -> bottom right order, by sorting each segment.
        segments
            .iter()
            .map(|(start, end)| {
                let mut v = [*start, *end];
                v.sort();
                let [start, end] = v;
                (start, end)
            })
            .collect()
    }
}
