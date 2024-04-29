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

    pub fn builder(&self) -> &DynamicStyleBuilder {
        &self.builder
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

    pub fn count_match(&self, node_states: &Vec<PseudoState>) -> usize {
        match &self.state {
            // Only consider pseudo themes that are specific to an inclusive substet of the themed element's pseudo states.
            // A theme for [Checked, Disabled] will apply to elements with [Checked, Disabled, FirstChild],
            // but will not apply to elements with [Checked] (because the theme targets more specific elements)
            // or [Checked, FirstChild] (because they are disjoint)
            Some(targeted_states) => match targeted_states
                .iter()
                .all(|state| node_states.contains(state))
            {
                true => targeted_states.len(),
                false => 0,
            },
            None => 0,
        }
    }
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: Component,
{
    context: PhantomData<C>,
    pseudo_themes: Vec<PseudoTheme>,
}

impl<C> Theme<C>
where
    C: Component,
{
    pub fn new(pseudo_themes: impl Into<Vec<PseudoTheme>>) -> Self {
        Self {
            context: PhantomData,
            pseudo_themes: pseudo_themes.into(),
        }
    }

    pub fn pseudo_themes(&self) -> &Vec<PseudoTheme> {
        &self.pseudo_themes
    }

    pub fn post_update() -> impl IntoSystemConfigs<()> {
        Theme::<C>::post_update_in(ThemeUpdate)
    }

    pub fn custom_post_update() -> impl IntoSystemConfigs<()> {
        Theme::<C>::post_update_in(CustomThemeUpdate)
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
        q_targets: Query<Entity, With<C>>,
        q_changed_targets: Query<Entity, (With<C>, Changed<PseudoStates>)>,
        mut q_removed_targets: RemovedComponents<PseudoStates>,
        mut commands: Commands,
    ) {
        for entity in &q_changed_targets {
            commands.entity(entity).refresh_theme::<C>();
        }

        for entity in q_removed_targets.read() {
            if q_targets.contains(entity) {
                commands.entity(entity).refresh_theme::<C>();
            }
        }
    }
}

#[derive(Default)]
pub struct ComponentThemePlugin<C>
where
    C: Component,
{
    context: PhantomData<C>,
    is_custom: bool,
}

impl<C> ComponentThemePlugin<C>
where
    C: Component,
{
    pub fn new() -> Self {
        Self {
            context: PhantomData,
            is_custom: false,
        }
    }

    /// Adds the theme update systems to the `CustomThemeUpdate` system set
    pub fn custom() -> Self {
        Self {
            context: PhantomData,
            is_custom: true,
        }
    }
}

impl<C> Plugin for ComponentThemePlugin<C>
where
    C: Component,
{
    fn build(&self, app: &mut App) {
        match self.is_custom {
            true => app.add_systems(PostUpdate, Theme::<C>::custom_post_update()),
            false => app.add_systems(PostUpdate, Theme::<C>::post_update()),
        };
    }
}
