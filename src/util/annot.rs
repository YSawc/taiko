#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loc(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Annot<T> {
    pub kind: T,
    pub loc: Loc,
}

impl<T> Annot<T> {
    pub fn new(kind: T, loc: Loc) -> Self {
        Annot { kind, loc }
    }

    pub fn loc(&self) -> Loc {
        self.loc
    }
}

impl Loc {
    pub fn merge(&self, loc: Loc) -> Self {
        use std::cmp::*;
        Loc(min(self.0, loc.0), max(self.1, loc.1))
    }

    pub fn new(start_pos: usize, end_pos: usize) -> Self {
        Loc(start_pos, end_pos)
    }
}
