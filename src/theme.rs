pub mod default_theme;
pub mod dynamic_style;
pub mod dynamic_style_attribute;
pub mod pseudo_state;
pub mod style_animation;

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{ui_commands::RefreshThemeExt, ui_style::StyleBuilder, widgets::WidgetLibraryUpdate};

use self::{
    default_theme::DefaultThemePlugin, dynamic_style::*, dynamic_style_attribute::*,
    pseudo_state::*, style_animation::*,
};

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                ThemeUpdate.after(WidgetLibraryUpdate),
                CustomThemeUpdate.after(ThemeUpdate),
            ),
        )
        .add_plugins((DefaultThemePlugin, DynamicStylePlugin));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ThemeUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct CustomThemeUpdate;

pub struct ThemeData {
    // Colors, floats, bools, strings (image/font path), handles
    // font data-> text styles -> per weight
    // should act as a cache for handles
}

pub struct ThemeBuilder {
    condition: Option<&'static [PseudoState]>,
    style_builder: fn(&mut StyleBuilder, &ThemeData),
    // static list variant
    // builder / theme data variant
    // WorldCallback variant
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: Component,
{
    context: PhantomData<C>,
    // TODO: Replace (PseudoStates->DynamicStyle) list with builders
    // Builder should store PseudoState list and builder fn
    styles: Vec<(Option<&'static [PseudoState]>, DynamicStyle)>,
}

impl<C> Theme<C>
where
    C: Component,
    Theme<C>: Default,
{
    pub fn update() -> impl IntoSystemConfigs<()> {
        Theme::<C>::update_in(ThemeUpdate)
    }

    pub fn update_in(set: impl SystemSet) -> impl IntoSystemConfigs<()> {
        (
            Theme::<C>::process_theme_update,
            Theme::<C>::process_updated_pseudo_states,
        )
            .in_set(set)
    }

    pub fn base_style(&self) -> Option<DynamicStyle> {
        self.styles
            .iter()
            .find(|(states, _)| states.is_none())
            .map(|(_, style)| style.clone())
    }

    pub fn style(&self, pseudo_states: &PseudoStates) -> Option<DynamicStyle> {
        let base_style = self.base_style();
        let override_style = self
            .styles
            .iter()
            .find(|(states, _)| states.is_some() && pseudo_states.in_state(states.unwrap()))
            .map(|(_, style)| style.clone());

        if let Some(override_style) = override_style {
            if let Some(base_style) = base_style {
                Some(base_style.merge(override_style))
            } else {
                Some(override_style)
            }
        } else {
            base_style
        }
    }

    fn process_theme_update(
        q_targets: Query<Entity, With<C>>,
        q_added_targets: Query<Entity, Added<C>>,
        q_removed_themes: RemovedComponents<Theme<C>>,
        q_changed_themes: Query<(Entity, &Theme<C>), Changed<Theme<C>>>,
        mut commands: Commands,
    ) {
        if q_removed_themes.len() > 0 || q_changed_themes.iter().count() > 0 {
            for entity in &q_targets {
                commands.entity(entity).refresh_theme();
            }
        } else {
            for entity in &q_added_targets {
                commands.entity(entity).refresh_theme();
            }
        }
    }

    fn process_updated_pseudo_states(
        q_changed_targets: Query<Entity, Changed<PseudoStates>>,
        mut q_removed_targets: RemovedComponents<PseudoStates>,
        mut commands: Commands,
    ) {
        for entity in &q_changed_targets {
            commands.entity(entity).refresh_theme();
        }

        for entity in q_removed_targets.read() {
            commands.entity(entity).refresh_theme();
        }
    }
}
