use std::collections::HashMap;

use crate::position::Position;

pub struct EdgeMap<T>(pub HashMap<Position, HashMap<Position, T>>);

impl<T> EdgeMap<T> {
    pub fn new() -> EdgeMap<T> {
        EdgeMap(HashMap::new())
    }

    pub fn get(&self, start: Position, end: Position) -> Option<&T> {
        self.0.get(&start)?.get(&end)
    }
}
