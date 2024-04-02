use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum PseudoState {
    #[default]
    Enabled,
    Disabled,
    Selected,
    Checked,
    Empty,
    FirstChild,
    NthChild(usize),
    LastChild,
    DirectionRow,
    DirectionColumn,
    OverflowX,
    OverflowY,
    Folded,
    Unfolded,
    Open,
    Closed,
    Custom(&'static str),
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct PseudoStates(Vec<PseudoState>);

impl PseudoStates {
    pub fn add(&mut self, state: PseudoState) {
        if !self.0.contains(&state) {
            self.0.push(state);
        }
    }

    pub fn remove(&mut self, state: PseudoState) {
        if self.0.contains(&state) {
            // Safe unwrap: checked in if
            self.0
                .remove(self.0.iter().position(|s| *s == state).unwrap());
        }
    }

    /// Compares the current list of PseudoStates with the provided `set`
    /// Returns `true` only when the contents of the set and the list is the same (order doesn't matter)
    pub fn in_state(&self, set: &[PseudoState]) -> bool {
        set.iter().all(|s| self.0.contains(s)) && self.0.iter().all(|s| set.contains(s))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
