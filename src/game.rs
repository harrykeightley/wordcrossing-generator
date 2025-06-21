use rand::prelude::*;
use serde::{
    Deserialize, Serialize, Serializer,
    ser::{SerializeMap, SerializeStruct},
};
use std::collections::{HashMap, HashSet};

use crate::{
    DistanceMap, TurnsMap,
    edge_map::EdgeMap,
    position::{Direction, Position},
};

#[derive(PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Entity {
    Wall,
    Letter(char),
    Nothing,
}

impl Entity {
    pub fn can_collide(&self) -> bool {
        match self {
            Entity::Wall => true,
            _ => false,
        }
    }
}

fn serialize_entities<S>(entities: &HashMap<Position, Entity>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut e = s.serialize_map(Some(entities.len()))?;
    for (k, v) in entities {
        e.serialize_entry(&k.to_key(), &v)?;
    }
    e.end()
}

#[derive(PartialEq, Eq, Clone, Serialize)]
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    #[serde(serialize_with = "serialize_entities")]
    pub entities: HashMap<Position, Entity>,
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            entities: HashMap::new(),
        }
    }

    pub fn all_positions(&self) -> Vec<Position> {
        let mut result: Vec<Position> = Vec::new();
        for row in 0..self.rows {
            for col in 0..self.cols {
                result.push(Position::new(row as isize, col as isize))
            }
        }
        result
    }

    pub fn add_entities(&mut self, entities: impl Iterator<Item = (Position, Entity)>) {
        entities.for_each(|(pos, entity)| {
            self.entities.insert(pos, entity);
        });
    }

    pub fn set_positions(&mut self, positions: Vec<Position>, entity: Entity) {
        self.add_entities(positions.into_iter().map(|p| (p, entity)))
    }

    pub fn randomise_walls(&mut self, min_area: f32, max_area: f32) {
        let roll: f32 = rand::random();
        let area = min_area + roll * (max_area - min_area);
        let wall_count = (area * self.rows as f32 * self.cols as f32).round() as usize;

        let mut rng = rand::rng();
        let mut positions = self.all_positions();
        positions.shuffle(&mut rng);
        let walls_to_be: Vec<Position> = positions.into_iter().take(wall_count).collect();
        self.set_positions(walls_to_be, Entity::Wall);
    }

    pub fn explore_section(&self, start: Position) -> HashSet<Position> {
        let mut visited: HashSet<Position> = HashSet::new();
        let mut grey: HashSet<Position> = HashSet::new();
        let mut queue: Vec<Position> = vec![start];

        if let Some(Entity::Wall) = self.entities.get(&start) {
            return visited;
        }

        while !queue.is_empty() {
            let node = queue.pop().unwrap();
            visited.insert(node);
            for neighbour in self.valid_neighbours(node) {
                if grey.contains(&neighbour) || visited.contains(&neighbour) {
                    continue;
                }
                if let Some(Entity::Wall) = self.entities.get(&neighbour) {
                    continue;
                }
                if !neighbour.is_within_bounds(self.rows as isize, self.cols as isize) {
                    continue;
                }
                queue.push(neighbour);
                grey.insert(neighbour);
            }
        }
        visited
    }

    pub fn find_connected_sections(&self) -> Vec<HashSet<Position>> {
        let mut result: Vec<HashSet<Position>> = Vec::new();
        let mut seen: HashSet<Position> = HashSet::new();

        for position in self.all_positions() {
            if seen.contains(&position) {
                continue;
            }
            let section = self.explore_section(position);
            if section.is_empty() {
                continue;
            }
            result.push(section.clone());
            seen = seen
                .union(&section)
                .into_iter()
                .map(|p| p.clone())
                .collect()
        }

        result
    }

    pub fn initialise_walls(&mut self) -> HashSet<Position> {
        self.randomise_walls(0.15, 0.5);
        let mut sections = self.find_connected_sections();
        // Sort by largest component
        sections.sort_by_key(|section| section.len());
        sections.reverse();

        // Wall off unreachable sections from the largest section.
        let unreachable: Vec<_> = sections.iter().skip(1).collect();
        for section in unreachable {
            self.set_positions(section.iter().cloned().collect(), Entity::Wall);
        }
        sections[0].clone()
    }

    pub fn free_space(&self) -> HashSet<Position> {
        self.find_connected_sections()
            .iter()
            .fold(HashSet::<Position>::new(), |space, section| {
                space.union(section).cloned().collect()
            })
    }

    pub fn valid_neighbours(&self, position: Position) -> Vec<Position> {
        position
            .neighbours()
            .into_iter()
            .filter(|&p| p.is_within_bounds(self.rows as isize, self.cols as isize))
            .collect()
    }

    pub fn generate_turns_map(&self) -> TurnsMap {
        let mut result: TurnsMap = EdgeMap::new();
        let free_space = self.free_space();

        // Initialise distances to selves as 0
        for position in free_space.iter().cloned() {
            // Set the distance to itself as 0
            let mut payload: HashMap<Position, (usize, Option<Direction>)> = HashMap::new();
            payload.insert(position, (0, None));
            result.0.insert(position, payload);
        }

        let mut changed = true;
        while changed {
            changed = false;
            for position in free_space.iter() {
                let turns = result.0.get(&position).unwrap();
                let mut next_turns = turns.clone();
                for neighbour in self.valid_neighbours(*position) {
                    if !free_space.contains(&neighbour) {
                        continue;
                    }
                    let neighbour_turns = result.0.get(&neighbour).unwrap();
                    for (destination, (neighbour_turn_count, direction)) in neighbour_turns {
                        let direction_to_neighbour = position.direction_to_position(neighbour);
                        let mut turn_count = *neighbour_turn_count;
                        if direction.clone() != direction_to_neighbour {
                            turn_count += 1;
                        }

                        if !turns.contains_key(destination)
                            || turn_count < turns.get(destination).unwrap().0
                        {
                            next_turns.insert(*destination, (turn_count, direction_to_neighbour));
                            changed = true;
                        }
                    }
                }
                result.0.insert(*position, next_turns);
            }
        }
        result
    }

    pub fn generate_distance_map(&self) -> DistanceMap {
        let mut result: DistanceMap = EdgeMap::new();
        let free_space = self.free_space();

        // Initialise distances to selves as 0
        for position in free_space.iter().cloned() {
            // Set the distance to itself as 0
            let mut payload: HashMap<Position, usize> = HashMap::new();
            payload.insert(position, 0);
            result.0.insert(position, payload);
        }

        let mut changed = true;
        while changed {
            changed = false;
            for position in free_space.iter() {
                let distances = result.0.get(&position).unwrap();
                let mut next_distances = distances.clone();
                for neighbour in self.valid_neighbours(*position) {
                    if !free_space.contains(&neighbour) {
                        continue;
                    }
                    let neighbour_distances = result.0.get(&neighbour).unwrap();
                    for (destination, distance) in neighbour_distances {
                        if !distances.contains_key(destination)
                            || distance + 1 < distances.get(destination).unwrap().clone()
                        {
                            next_distances.insert(destination.clone(), distance + 1);
                            changed = true;
                        }
                    }
                }
                result.0.insert(*position, next_distances);
            }
        }
        result
    }

    pub fn visualise(&self) {
        let bar = "#".repeat(self.cols);
        for row in 0..self.rows {
            let mut line = String::new();
            for col in 0..self.cols {
                let position = Position::new(row as isize, col as isize);
                let letter = match self.entities.get(&position) {
                    Some(Entity::Wall) => '#',
                    Some(Entity::Letter(a)) => a.clone(),
                    Some(Entity::Nothing) => ' ',
                    None => ' ',
                };
                line.push(letter)
            }
            println!("{}", line);
        }
        println!("{}", bar);
    }
}

#[derive(Serialize)]
pub struct Level {
    pub start: Position,
    pub goal: Position,
    pub words: Vec<String>,
    pub grid: Grid,
}

impl Level {
    pub fn visualise(&self) {
        let bar = "=".repeat(self.grid.cols);
        println!("{}", bar);
        for row in 0..self.grid.rows {
            let mut line = String::new();
            for col in 0..self.grid.cols {
                let position = Position::new(row as isize, col as isize);
                if self.start == position {
                    line += "S"
                } else if self.goal == position {
                    line += "G"
                } else {
                    let letter = match self.grid.entities.get(&position) {
                        Some(Entity::Wall) => '#',
                        Some(Entity::Letter(a)) => a.clone(),
                        Some(Entity::Nothing) => ' ',
                        None => ' ',
                    };
                    line.push(letter)
                }
            }
            println!("{}", line);
        }
        println!("{}", bar);
        println!("Solution: {:?}", self.words);
    }
}
