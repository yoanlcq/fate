use std::cell::Cell;

#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub origin: SplitOrigin,
    pub unit: SplitUnit,
    pub value: Cell<f32>,
    pub direction: SplitDirection,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitOrigin {
    LeftOrBottom, Middle, RightOrTop,    
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitUnit {
    Ratio, Px,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, Vertical,
}
