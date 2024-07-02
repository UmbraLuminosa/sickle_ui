pub mod dynamic_style;
pub mod dynamic_style_attribute;
pub mod icons;
pub mod pseudo_state;
pub mod style_animation;
pub mod theme_colors;
pub mod theme_data;
pub mod theme_spacing;
pub mod typography;

use std::{
    any::{type_name, TypeId},
    collections::HashSet,
    marker::PhantomData,
};

use bevy::{prelude::*, ui::UiSystem};

use dynamic_style::{DynamicStyle, DynamicStylePlugin};
use pseudo_state::{AutoPseudoStatePlugin, PseudoState, PseudoStates};
use theme_data::ThemeData;

use crate::{prelude::UiBuilder, ui_commands::RefreshThemeExt, ui_style::builder::StyleBuilder};

pub mod prelude {
    pub use super::{
        dynamic_style::{
            ContextStyleAttribute, DynamicStyle, DynamicStyleEnterState, DynamicStylePostUpdate,
        },
        dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController},
        icons::IconData,
        pseudo_state::{
            FlexDirectionToPseudoState, HierarchyToPseudoState, PseudoState, PseudoStates,
            VisibilityToPseudoState,
        },
        style_animation::{
            AnimationLoop, AnimationSettings, AnimationState, InteractionStyle,
            LoopedAnimationConfig,
        },
        theme_colors::{Accent, Container, On, Surface},
        theme_data::{Contrast, Scheme, ThemeData},
        typography::{FontScale, FontStyle, FontType, SizedFont},
        ComponentThemePlugin, CustomThemeUpdate, DefaultTheme, DynamicStyleBuilder, PseudoTheme,
        Theme, ThemeUpdate, UiContext,
    };
}

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (ThemeUpdate, CustomThemeUpdate.after(ThemeUpdate)).before(UiSystem::Layout),
        )
        .init_resource::<ThemeData>()
        .init_resource::<ThemeRegistry>()
        .add_plugins((AutoPseudoStatePlugin, DynamicStylePlugin));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ThemeUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct CustomThemeUpdate;

#[derive(Clone, Debug)]
pub enum DynamicStyleBuilder<C> {
    Static(DynamicStyle),
    StyleBuilder(fn(&mut StyleBuilder, &ThemeData)),
    ContextStyleBuilder(fn(&mut StyleBuilder, &C, &ThemeData)),
    WorldStyleBuilder(fn(&mut StyleBuilder, Entity, &C, &World)),
    InfoWorldStyleBuilder(
        fn(&mut StyleBuilder, Option<Entity>, &Option<Vec<PseudoState>>, Entity, &C, &World),
    ),
}

impl<C> From<StyleBuilder> for DynamicStyleBuilder<C> {
    fn from(value: StyleBuilder) -> Self {
        Self::Static(value.into())
    }
}

impl<C> From<DynamicStyle> for DynamicStyleBuilder<C> {
    fn from(value: DynamicStyle) -> Self {
        Self::Static(value)
    }
}

#[derive(Clone, Debug)]
pub struct PseudoTheme<C> {
    state: Option<Vec<PseudoState>>,
    builder: DynamicStyleBuilder<C>,
}

impl<C> PseudoTheme<C> {
    pub fn new(
        state: impl Into<Option<Vec<PseudoState>>>,
        theme: impl Into<DynamicStyleBuilder<C>>,
    ) -> Self {
        Self {
            state: state.into(),
            builder: theme.into(),
        }
    }

    pub fn state(&self) -> &Option<Vec<PseudoState>> {
        &self.state
    }

    pub fn builder(&self) -> &DynamicStyleBuilder<C> {
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

    pub fn deferred_context(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(&mut StyleBuilder, &C, &ThemeData),
    ) -> Self {
        Self {
            state: state.into(),
            builder: DynamicStyleBuilder::ContextStyleBuilder(builder),
        }
    }

    pub fn deferred_world(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(&mut StyleBuilder, Entity, &C, &World),
    ) -> Self {
        Self {
            state: state.into(),
            builder: DynamicStyleBuilder::WorldStyleBuilder(builder),
        }
    }

    pub fn deferred_info_world(
        state: impl Into<Option<Vec<PseudoState>>>,
        builder: fn(
            &mut StyleBuilder,
            Option<Entity>,
            &Option<Vec<PseudoState>>,
            Entity,
            &C,
            &World,
        ),
    ) -> Self {
        Self {
            state: state.into(),
            builder: DynamicStyleBuilder::InfoWorldStyleBuilder(builder),
        }
    }

    pub fn is_base_theme(&self) -> bool {
        match &self.state {
            Some(list) => list.is_empty(),
            None => true,
        }
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

pub trait UiContext {
    fn get(&self, _target: &str) -> Result<Entity, String> {
        Err(format!(
            "{} has no UI contexts",
            std::any::type_name::<Self>()
        ))
    }

    /// These are the contexts cleared by the parent theme when no DynamicStyle
    /// is placed to them.
    ///
    /// By default this is the full list of contexts.
    ///
    /// Warning: If a context is a sub-widget with its own theme, it should not
    /// be included in the cleared contexts, nor should it be used for placement
    /// from the main entity. The behavior is undefined.
    fn cleared_contexts(&self) -> Vec<&'static str> {
        self.contexts()
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![]
    }
}

pub trait DefaultTheme: Sized + Component + UiContext {
    fn default_theme() -> Option<Theme<Self>> {
        None
    }
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: DefaultTheme,
{
    context: PhantomData<C>,
    pseudo_themes: Vec<PseudoTheme<C>>,
}

impl<C> Theme<C>
where
    C: DefaultTheme,
{
    pub fn new(pseudo_themes: impl Into<Vec<PseudoTheme<C>>>) -> Self {
        Self {
            context: PhantomData,
            pseudo_themes: pseudo_themes.into(),
        }
    }

    pub fn pseudo_themes(&self) -> &Vec<PseudoTheme<C>> {
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

    fn process_theme_update(
        q_targets: Query<Entity, With<C>>,
        q_added_targets: Query<Entity, Added<C>>,
        q_removed_themes: RemovedComponents<Theme<C>>,
        q_changed_themes: Query<Entity, Changed<Theme<C>>>,
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

pub trait InsertThemedComponentExt {
    /// Inserts `C` as a component to the entity and checks if [`ComponentThemePlugin<C>`](ComponentThemePlugin)
    /// was added to the app.
    fn insert_themed_component<C: DefaultTheme + Component>(&mut self, component: C) -> &mut Self;
}

impl InsertThemedComponentExt for UiBuilder<'_, Entity> {
    fn insert_themed_component<C: DefaultTheme + Component>(&mut self, component: C) -> &mut Self {
        self.insert(component);
        self.commands().add(|world: &mut World| {
            if !world.resource::<ThemeRegistry>().contains_by_id(TypeId::of::<C>()) {
                warn!("themed component {} was not registered; add its ComponentThemePlugin to your app", type_name::<C>());
            }
        });
        self
    }
}

/// Tracks all the themes that have been registered with [`ComponentThemePlugin`].
///
/// This can be used to check if a theme's plugin is missing.
#[derive(Resource, Default, Debug)]
pub struct ThemeRegistry {
    themes: HashSet<TypeId>,
}

impl ThemeRegistry {
    fn new_with<C: 'static>() -> Self {
        let mut registry = Self::default();
        registry.add::<C>();
        registry
    }

    fn add<C: 'static>(&mut self) {
        self.themes.insert(TypeId::of::<C>());
    }

    /// Returns `true` if the theme `C` has been registered.
    pub fn contains<C: 'static>(&self) -> bool {
        self.contains_by_id(TypeId::of::<C>())
    }

    /// Returns `true` if the theme `id` has been registered.
    pub fn contains_by_id(&self, id: TypeId) -> bool {
        self.themes.contains(&id)
    }
}

#[derive(Default)]
pub struct ComponentThemePlugin<C>
where
    C: DefaultTheme,
{
    context: PhantomData<C>,
    is_custom: bool,
}

impl<C> ComponentThemePlugin<C>
where
    C: DefaultTheme,
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
    C: DefaultTheme,
{
    fn build(&self, app: &mut App) {
        if let Some(mut registry) = app.world.get_resource_mut::<ThemeRegistry>() {
            registry.add::<C>();
        } else {
            app.insert_resource(ThemeRegistry::new_with::<C>());
        }

        match self.is_custom {
            true => app.add_systems(PostUpdate, Theme::<C>::custom_post_update()),
            false => app.add_systems(PostUpdate, Theme::<C>::post_update()),
        };
    }
}
