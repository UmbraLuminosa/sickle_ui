use bevy::prelude::*;

use crate::widgets::{
    context_menu::ContextMenu, docking_zone::DockingZoneHighlight, prelude::Checkbox,
};

use super::Theme;

pub struct DefaultThemePlugin;

impl Plugin for DefaultThemePlugin {
    // TODO: Inject default theme to root nodes when cfg flag enabled
    fn build(&self, app: &mut App) {
        app.add_systems(Update, inject_default_theme);
    }
}

fn inject_default_theme(
    q_root_nodes: Query<Entity, (With<Node>, Without<Parent>, Without<DefaultTheme>)>,
    mut commands: Commands,
) {
    for entity in &q_root_nodes {
        commands.entity(entity).insert(ThemeBundle::default());
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct DefaultTheme;

#[derive(Bundle, Debug, Default)]
pub struct ThemeBundle {
    marker: DefaultTheme,
    checkbox: Theme<Checkbox>,
    context_menu: Theme<ContextMenu>,
    docking_zone_highlight: Theme<DockingZoneHighlight>,
}
