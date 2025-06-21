use std::collections::HashMap;

use crate::position::Position;

/// A pairwise mapping of two points within the grid, to some data type T.
pub struct EdgeMap<T>(pub HashMap<Position, HashMap<Position, T>>);

impl<T> EdgeMap<T> {
    pub fn new() -> EdgeMap<T> {
        EdgeMap(HashMap::new())
    }

    pub fn get(&self, start: Position, end: Position) -> Option<&T> {
        self.0.get(&start)?.get(&end)
    }
}
