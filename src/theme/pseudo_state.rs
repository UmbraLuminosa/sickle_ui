use bevy::{prelude::*, render::view::VisibilitySystems};
use serde::{Deserialize, Serialize};

use super::ThemeUpdate;

pub struct AutoPseudoStatePlugin;

impl Plugin for AutoPseudoStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            track_node_visibility
                .after(VisibilitySystems::VisibilityPropagate)
                .before(ThemeUpdate),
        );
    }
}

fn track_node_visibility(
    mut q_nodes: Query<
        (&mut PseudoStates, &Visibility, &InheritedVisibility),
        Or<(Changed<Visibility>, Changed<InheritedVisibility>)>,
    >,
) {
    for (mut state, visibility, inherited) in &mut q_nodes {
        let visible = visibility == Visibility::Visible
            || (inherited.get() && visibility == Visibility::Inherited);

        if visible {
            state.add(PseudoState::Visible);
        } else {
            state.remove(PseudoState::Visible);
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Reflect, Serialize, Deserialize,
)]
pub enum PseudoState {
    #[default]
    Enabled,
    Disabled,
    Visible,
    Selected,
    Checked,
    Empty,
    FirstChild,
    NthChild(usize),
    LastChild,
    Even,
    Odd,
    DirectionRow,
    DirectionColumn,
    OverflowX,
    OverflowY,
    Folded,
    Unfolded,
    Open,
    Closed,
    Error,
    Custom(&'static str),
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct PseudoStates(Vec<PseudoState>);

impl From<Vec<PseudoState>> for PseudoStates {
    fn from(value: Vec<PseudoState>) -> Self {
        let mut uniques: Vec<PseudoState> = Vec::with_capacity(value.len());
        for val in value {
            if !uniques.contains(&val) {
                uniques.push(val);
            }
        }

        Self(uniques)
    }
}

impl PseudoStates {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn has(&self, state: PseudoState) -> bool {
        self.0.contains(&state)
    }

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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self) -> &Vec<PseudoState> {
        &self.0
    }
}
