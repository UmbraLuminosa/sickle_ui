pub mod color_palette;
pub mod default_theme;
pub mod dynamic_style;
pub mod dynamic_style_attribute;
pub mod pseudo_state;
pub mod style_animation;

use std::marker::PhantomData;

use bevy::{prelude::*, ui::UiSystem};

use crate::{ui_commands::RefreshThemeExt, ui_style::StyleBuilder};

use self::{
    default_theme::DefaultThemePlugin, dynamic_style::*, dynamic_style_attribute::*,
    pseudo_state::*, style_animation::*,
};

pub struct ThemePlugin;

// TODO: move to post updates
impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (ThemeUpdate, CustomThemeUpdate.after(ThemeUpdate)).before(UiSystem::Layout),
        )
        .init_resource::<ThemeData>()
        .add_plugins((DefaultThemePlugin, DynamicStylePlugin));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ThemeUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct CustomThemeUpdate;

#[derive(Resource, Clone, Debug, Reflect)]
pub struct ThemeData {
    pub background_color: Color,
    // Colors, floats, bools, strings (image/font path), handles
    // font data-> text styles -> per weight
    // should act as a cache for handles
}

impl Default for ThemeData {
    fn default() -> Self {
        Self {
            background_color: Color::BLUE,
        }
    }
}

// TODO: Add support for enter/exit pseudo state animations
#[derive(Clone, Debug)]
pub enum DynamicStyleBuilder {
    Static(DynamicStyle),
    StyleBuilder(fn(&mut StyleBuilder, &ThemeData)),
    WorldStyleBuilder(fn(&mut StyleBuilder, &mut World)),
}

impl From<StyleBuilder> for DynamicStyleBuilder {
    fn from(value: StyleBuilder) -> Self {
        Self::Static(value.into())
    }
}

impl From<DynamicStyle> for DynamicStyleBuilder {
    fn from(value: DynamicStyle) -> Self {
        Self::Static(value)
    }
}

// TODO: investigate proper From implementation so it accepts static functions
impl From<fn(&mut StyleBuilder, &ThemeData)> for DynamicStyleBuilder {
    fn from(value: fn(&mut StyleBuilder, &ThemeData)) -> Self {
        Self::StyleBuilder(value)
    }
}

impl From<fn(&mut StyleBuilder, &mut World)> for DynamicStyleBuilder {
    fn from(value: fn(&mut StyleBuilder, &mut World)) -> Self {
        Self::WorldStyleBuilder(value)
    }
}

#[derive(Clone, Debug)]
pub struct PseudoTheme {
    state: Option<Vec<PseudoState>>,
    builder: DynamicStyleBuilder,
}

impl PseudoTheme {
    pub fn new(
        state: impl Into<Option<Vec<PseudoState>>>,
        theme: impl Into<DynamicStyleBuilder>,
    ) -> Self {
        Self {
            state: state.into(),
            builder: theme.into(),
        }
    }

    pub fn build(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(&mut StyleBuilder),
    ) -> Self {
        let mut style_builder = StyleBuilder::new();
        builder(&mut style_builder);

        Self {
            state: state.into(),
            builder: style_builder.into(),
        }
    }

    pub fn deferred(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(&mut StyleBuilder, &ThemeData),
    ) -> Self {
        Self {
            state: state.into(),
            builder: DynamicStyleBuilder::StyleBuilder(builder),
        }
    }

    pub fn deferred_world(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(&mut StyleBuilder, &mut World),
    ) -> Self {
        Self {
            state: state.into(),
            builder: DynamicStyleBuilder::WorldStyleBuilder(builder),
        }
    }

    pub fn is_base_theme(&self) -> bool {
        self.state.is_none()
    }

    pub fn for_state(&self, pseudo_states: &PseudoStates) -> bool {
        match &self.state {
            Some(states) => pseudo_states.in_state(states),
            None => false,
        }
    }
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: Component,
{
    context: PhantomData<C>,
    styles: Vec<PseudoTheme>,
}

impl<C> Theme<C>
where
    C: Component,
{
    pub fn new(styles: impl Into<Vec<PseudoTheme>>) -> Self {
        Self {
            context: PhantomData,
            styles: styles.into(),
        }
    }
}

impl<C> Theme<C>
where
    C: Component,
{
    pub fn base_builder(&self) -> Option<DynamicStyleBuilder> {
        self.styles
            .iter()
            .find(|pseudo_theme| pseudo_theme.is_base_theme())
            .map(|pseudo_theme| pseudo_theme.builder.clone())
    }

    pub fn builder_for(&self, pseudo_states: &PseudoStates) -> Option<DynamicStyleBuilder> {
        self.styles
            .iter()
            .find(|pseudo_theme| pseudo_theme.for_state(pseudo_states))
            .map(|pseudo_theme| pseudo_theme.builder.clone())
    }

    pub fn post_update() -> impl IntoSystemConfigs<()> {
        Theme::<C>::post_update_in(ThemeUpdate)
    }

    pub fn post_update_in(set: impl SystemSet) -> impl IntoSystemConfigs<()> {
        (
            Theme::<C>::process_theme_update,
            Theme::<C>::process_updated_pseudo_states,
        )
            .in_set(set)
    }

    // TODO: Implement ui_builder.themed_root(theme) extension?
    fn process_theme_update(
        q_targets: Query<Entity, With<C>>,
        q_added_targets: Query<Entity, Added<C>>,
        q_removed_themes: RemovedComponents<Theme<C>>,
        q_changed_themes: Query<(Entity, &Theme<C>), Changed<Theme<C>>>,
        theme_data: Res<ThemeData>,
        mut commands: Commands,
    ) {
        if theme_data.is_changed()
            || q_removed_themes.len() > 0
            || q_changed_themes.iter().count() > 0
        {
            for entity in &q_targets {
                commands.entity(entity).refresh_theme::<C>();
            }
        } else {
            for entity in &q_added_targets {
                commands.entity(entity).refresh_theme::<C>();
            }
        }
    }

    fn process_updated_pseudo_states(
        q_changed_targets: Query<Entity, Changed<PseudoStates>>,
        mut q_removed_targets: RemovedComponents<PseudoStates>,
        mut commands: Commands,
    ) {
        for entity in &q_changed_targets {
            commands.entity(entity).refresh_theme::<C>();
        }

        for entity in q_removed_targets.read() {
            commands.entity(entity).refresh_theme::<C>();
        }
    }
}
