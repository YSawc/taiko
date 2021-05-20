#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loc(pub usize, pub usize, pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Annot<T> {
    pub value: T,
    pub loc: Loc,
}

impl<T> Annot<T> {
    pub fn new(value: T, loc: Loc) -> Self {
        Annot { value, loc }
    }
}

impl Loc {
    pub fn merge(&self, loc: Loc) -> Loc {
        use std::cmp::*;
        Loc(
            min(self.0, loc.0),
            max(self.1, loc.1),
            min(self.2, loc.2),
            max(self.3, loc.3),
        )
    }
}
