use std::ops;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub struct Position {
    pub row: isize,
    pub col: isize,
}

impl Position {
    pub fn new(row: isize, col: isize) -> Position {
        Position { row, col }
    }

    pub fn to_key(&self) -> String {
        let mut res = self.row.to_string();
        res.push('_');
        res += self.col.to_string().as_str();
        res
    }

    pub fn is_within_bounds(&self, rows: isize, cols: isize) -> bool {
        self.row >= 0 && self.row < rows && self.col >= 0 && self.col < cols
    }

    pub fn step_in_direction(&self, direction: Direction) -> Position {
        self.clone() + direction_delta(direction)
    }

    pub fn manhattan_distance(&self, position: Position) -> usize {
        self.row.abs_diff(position.row) + self.col.abs_diff(position.col)
    }

    pub fn direction_to_position(&self, position: Position) -> Option<Direction> {
        for direction in vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            if self.step_in_direction(direction) == position {
                return Some(direction);
            }
        }
        None
    }

    pub fn neighbours(&self) -> Vec<Position> {
        let directions: Vec<Direction> = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        directions
            .iter()
            .map(|d| self.step_in_direction(d.clone()))
            .collect()
    }
}

impl ops::Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position::new(self.row + rhs.row, self.col + rhs.col)
    }
}

impl ops::Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        return self + (-rhs);
    }
}

impl ops::Neg for Position {
    type Output = Position;

    fn neg(self) -> Self::Output {
        Position::new(-self.row, -self.col)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn direction_delta(direction: Direction) -> Position {
    match direction {
        Direction::Up => Position::new(-1, 0),
        Direction::Down => Position::new(1, 0),
        Direction::Left => Position::new(0, -1),
        Direction::Right => Position::new(0, 1),
    }
}
