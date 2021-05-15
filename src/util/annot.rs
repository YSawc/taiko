#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loc(pub usize, pub usize, pub usize);

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
