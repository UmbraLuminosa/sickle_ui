use bevy::prelude::*;

pub struct DefaultThemePlugin;

impl Plugin for DefaultThemePlugin {
    // TODO: Inject default theme to root nodes when cfg flag enabled
    fn build(&self, _app: &mut App) {}
}

pub struct DefaultThemeBundle {}
