use core::panic;
use std::{marker::PhantomData, num::NonZeroU16};

use bevy::{ecs::system::EntityCommand, prelude::*};
use sickle_math::{ease::Ease, lerp::Lerp};

use crate::{
    ui_commands::InsertDynamicStyleExt,
    ui_style::{SetBackgroundColorExt, UiStyle, UiStyleExt},
    widgets::{floating_panel::FloatingPanelFoldButton, WidgetLibraryUpdate},
    FluxInteraction, FluxInteractionStopwatch,
};

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, ThemeUpdate.after(WidgetLibraryUpdate))
            .configure_sets(Update, DynamicStyleUpdate.after(ThemeUpdate))
            .add_systems(
                Update,
                (
                    update_dynamic_styles,
                    update_pseudo_dynamic_styles,
                    update_flux_dynamic_styles,
                    update_pseudo_flux_dynamic_styles,
                )
                    .in_set(DynamicStyleUpdate),
            )
            .add_systems(
                Update,
                apply_theme::<FloatingPanelFoldButton>.in_set(ThemeUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ThemeUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DynamicStyleUpdate;

// These are dynamic styles with only inert values
fn update_dynamic_styles(
    q_styles: Query<
        (Entity, &DynamicStyle),
        (
            Without<PseudoDynamicStyle>,
            Without<FluxDynamicStyle>,
            Without<PseudoFluxDynamicStyle>,
            Changed<DynamicStyle>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, style) in &q_styles {
        style.apply(None, entity, commands.reborrow());
    }
}

// These are styles that have no interactivity, just inert styling based on pseudo-state
fn update_pseudo_dynamic_styles(
    q_styles: Query<
        (Entity, &DynamicStyle, &PseudoState),
        (
            With<PseudoDynamicStyle>,
            Or<(Changed<DynamicStyle>, Changed<PseudoState>)>,
        ),
    >,
    mut q_removed_pseudo_states: RemovedComponents<PseudoState>,
    q_pseudo_styles: Query<(Entity, &DynamicStyle), With<PseudoDynamicStyle>>,
    mut commands: Commands,
) {
    for (entity, style, state) in &q_styles {
        style.apply(Some(*state), entity, commands.reborrow());
    }

    for entity in q_removed_pseudo_states.read() {
        let Ok((entity, style)) = q_pseudo_styles.get(entity) else {
            continue;
        };

        style.apply(None, entity, commands.reborrow());
    }
}

// These are styles that react to interactions, but are not based on pseudo-states
fn update_flux_dynamic_styles(
    q_styles: Query<
        (
            Entity,
            &DynamicStyle,
            &FluxInteraction,
            &FluxInteractionStopwatch,
        ),
        (
            With<FluxDynamicStyle>,
            Or<(
                Changed<DynamicStyle>,
                Changed<FluxInteraction>,
                Changed<FluxInteractionStopwatch>,
            )>,
        ),
    >,
    mut commands: Commands,
) {
}

fn update_pseudo_flux_dynamic_styles(q_styles: Query<&DynamicStyle>, mut commands: Commands) {}

pub fn apply_theme<C>(
    q_targets: Query<(Entity, Ref<C>), With<C>>,
    q_added_targets: Query<Entity, Added<C>>,
    q_changed_themes: Query<(Entity, &Theme<C>), Changed<Theme<C>>>,
    q_removed_themes: RemovedComponents<Theme<C>>,
    q_theme: Query<(Entity, &Theme<C>)>,
    q_parents: Query<&Parent>,
    mut commands: Commands,
) where
    C: Component,
    Theme<C>: Default,
{
    if q_removed_themes.len() > 0 {
        // Reprocess all theme targets
        for (entity, target) in &q_targets {
            let own_theme = q_theme.get(entity);
            // New targets need to try setting their own theme
            if target.is_added() && own_theme.is_ok() {
                // Safe unwrap: checked already ^^^^^
                let (_, theme) = own_theme.unwrap();
                commands.entity(entity).insert_theme(theme);
            } else if own_theme.is_err() {
                // Only search for a new parent theme if entity doesn't have its own
                let mut found_theme = false;
                for parent in q_parents.iter_ancestors(entity) {
                    if let Ok((_, theme)) = q_theme.get(parent) {
                        commands.entity(entity).insert_theme(theme);

                        found_theme = true;
                        break;
                    }
                }

                if !found_theme {
                    let default_theme = Theme::<C>::default();
                    commands.entity(entity).insert_theme(&default_theme);
                }
            }
        }

        return;
    }

    for entity in &q_added_targets {
        if let Ok((_, theme)) = q_theme.get(entity) {
            commands.entity(entity).insert_theme(theme);
        } else {
            let mut found_theme = false;
            for parent in q_parents.iter_ancestors(entity) {
                if let Ok((_, theme)) = q_theme.get(parent) {
                    commands.entity(entity).insert_theme(theme);

                    found_theme = true;
                    break;
                }
            }

            if !found_theme {
                let default_theme = Theme::<C>::default();

                commands.entity(entity).insert_theme(&default_theme);
            }
        }
    }

    for (theme_entity, theme) in &q_changed_themes {
        for (entity, target) in &q_targets {
            // Processed separately above
            if target.is_added() {
                continue;
            }

            if entity == theme_entity {
                commands.entity(entity).insert_theme(theme);
            } else {
                for parent in q_parents.iter_ancestors(entity) {
                    if let Ok((other_theme_entity, _)) = q_theme.get(parent) {
                        if other_theme_entity == theme_entity {
                            commands.entity(entity).insert_theme(theme);
                        }
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub enum PseudoState {
    #[default]
    Enabled,
    Disabled,
    Selected,
    Checked,
    Empty,
}

#[derive(Component, Debug)]
pub struct Theme<C>
where
    C: Component,
{
    context: PhantomData<C>,
    style: DynamicStyle,
}

impl<C> Theme<C>
where
    C: Component,
    Theme<C>: Default,
{
    pub fn style(&self) -> DynamicStyle {
        self.style.clone()
    }
}

// TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, Default)]
pub enum AnimationProgress {
    #[default]
    Start,
    Inbetween(f32, usize),
    End,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AnimationLoop {
    #[default]
    Continous,
    Times(NonZeroU16),
    PingPongContinous,
    PingPong(NonZeroU16),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StyleAnimationConfig {
    duration: f32,
    easing: Ease,
    delay: Option<f32>,
    loop_type: Option<AnimationLoop>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StyleAnimation {
    pointer_enter: Option<StyleAnimationConfig>,
    pointer_leave: Option<StyleAnimationConfig>,
    press: Option<StyleAnimationConfig>,
    release: Option<StyleAnimationConfig>,
    cancel: Option<StyleAnimationConfig>,
    focus: Option<StyleAnimationConfig>,
    focus_lost: Option<StyleAnimationConfig>,
    progress: AnimationProgress,
}

macro_rules! animation_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            easing: Ease,
            delay: Option<f32>,
            loop_type: Option<AnimationLoop>,
        ) -> &mut StyleAnimation {
            let config = StyleAnimationConfig {
                duration,
                easing,
                delay,
                loop_type,
            };
            self.$setter = Some(config);

            self
        }
    };
}

impl StyleAnimation {
    pub fn new() -> Self {
        Self { ..default() }
    }

    animation_setter!(pointer_enter);
    animation_setter!(pointer_leave);
    animation_setter!(press);
    animation_setter!(release);
    animation_setter!(cancel);
    animation_setter!(focus);
    animation_setter!(focus_lost);

    pub fn all(
        &mut self,
        duration: f32,
        easing: Ease,
        delay: Option<f32>,
        loop_type: Option<AnimationLoop>,
    ) -> &mut StyleAnimation {
        let config = StyleAnimationConfig {
            duration,
            easing,
            delay,
            loop_type,
        };
        self.pointer_enter = Some(config);
        self.pointer_leave = Some(config);
        self.press = Some(config);
        self.release = Some(config);
        self.cancel = Some(config);
        self.focus = Some(config);
        self.focus_lost = Some(config);

        self
    }

    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DynamicStyleAttribute<T: Lerp + Default + Clone + Copy + PartialEq> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
    animation: Option<StyleAnimation>,
    current: T,
    dirty: bool,
}

impl<T: Lerp + Default + Clone + Copy + PartialEq> DynamicStyleAttribute<T> {
    pub fn new(base: T) -> Self {
        Self {
            base,
            current: base,
            dirty: true,
            ..default()
        }
    }

    pub fn base(&mut self, value: T) -> &mut Self {
        self.base = value;
        self
    }

    pub fn hover(&mut self, value: T) -> &mut Self {
        self.hover = value.into();
        self
    }

    pub fn press(&mut self, value: T) -> &mut Self {
        self.press = value.into();
        self
    }

    pub fn cancel(&mut self, value: T) -> &mut Self {
        self.cancel = value.into();
        self
    }

    pub fn focus(&mut self, value: T) -> &mut Self {
        self.focus = value.into();
        self
    }

    pub fn set_animation(&mut self, animation: StyleAnimation) -> &mut Self {
        self.animation = Some(animation);
        self
    }

    pub fn animate(&mut self) -> &mut StyleAnimation {
        let animation = StyleAnimation::new();
        self.animation = Some(animation);

        let Some(ref mut animation) = self.animation else {
            unreachable!();
        };

        animation
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn need_flux_interaction(&self) -> bool {
        self.hover.is_some()
            || self.press.is_some()
            || self.cancel.is_some()
            || self.focus.is_some()
    }

    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
    }
}

struct CustomDynamicStyle {
    callback: fn(f32, Entity, &mut World),
    current_value: f32,
}

impl EntityCommand for CustomDynamicStyle {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(self.current_value, id, world);
    }
}

#[derive(Clone, Debug)]
pub enum DynamicStyleAttributes {
    BackgroundColor(DynamicStyleAttribute<Color>),
    Custom(fn(f32, Entity, &mut World), DynamicStyleAttribute<f32>),
}

impl PartialEq for DynamicStyleAttributes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
            (Self::Custom(l0, _), Self::Custom(r0, _)) => l0 == r0,
            _ => false,
        }
    }
}

impl DynamicStyleAttributes {
    pub fn need_flux_interaction(&self) -> bool {
        match self {
            DynamicStyleAttributes::BackgroundColor(attr) => attr.need_flux_interaction(),
            DynamicStyleAttributes::Custom(_, attr) => attr.need_flux_interaction(),
        }
    }

    pub fn apply<'a>(&'a self, ui_style: &'a mut UiStyle<'a>) {
        match self {
            DynamicStyleAttributes::BackgroundColor(attr) => {
                ui_style.background_color(attr.current);
            }
            DynamicStyleAttributes::Custom(callback, attr) => {
                ui_style.entity_commands().add(CustomDynamicStyle {
                    callback: *callback,
                    current_value: attr.current,
                });
            }
        };
    }
}

pub struct DynamicStyleBuilder {
    attributes: Vec<DynamicStyleAttributes>,
}

macro_rules! style_builder {
    ($func_name:ident, $variant:path, $type:ident) => {
        fn $func_name(&mut self, base: $type) -> &mut DynamicStyleAttribute<$type> {
            let attribute = DynamicStyleAttribute::new(base);
            let variant = $variant(attribute);

            if self.attributes.contains(&variant) {
                let Some(&mut $variant(ref mut unwrapped_attr)) =
                    self.attributes.iter_mut().find(|item| **item == variant)
                else {
                    unreachable!();
                };

                unwrapped_attr.base(base);
                return unwrapped_attr;
            }

            self.attributes.push($variant(attribute));

            let Some(&mut $variant(ref mut unwrapped_attr)) = self.attributes.last_mut() else {
                unreachable!();
            };

            unwrapped_attr
        }
    };
}

impl DynamicStyleBuilder {
    fn new() -> Self {
        Self { attributes: vec![] }
    }

    style_builder!(
        background_color,
        DynamicStyleAttributes::BackgroundColor,
        Color
    );
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PseudoDynamicStyle;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FluxDynamicStyle;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PseudoFluxDynamicStyle;

#[derive(Component, Clone, Debug)]
pub struct DynamicStyle {
    base: Vec<DynamicStyleAttributes>,
    enabled: Option<Vec<DynamicStyleAttributes>>,
    disabled: Option<Vec<DynamicStyleAttributes>>,
    selected: Option<Vec<DynamicStyleAttributes>>,
    checked: Option<Vec<DynamicStyleAttributes>>,
    empty: Option<Vec<DynamicStyleAttributes>>,
}

impl Default for DynamicStyle {
    fn default() -> Self {
        Self {
            base: Default::default(),
            enabled: Default::default(),
            disabled: Default::default(),
            selected: Default::default(),
            checked: Default::default(),
            empty: Default::default(),
        }
    }
}

macro_rules! pseudo_state_setter {
    ($setter:ident) => {
        pub fn $setter(
            &mut self,
            style_builder: impl FnOnce(&mut DynamicStyleBuilder),
        ) -> &mut Self {
            let mut base_builder = DynamicStyleBuilder::new();
            style_builder(&mut base_builder);

            self.$setter = base_builder.attributes.into();
            self
        }
    };
}

impl DynamicStyle {
    pub fn build(style_builder: impl FnOnce(&mut DynamicStyleBuilder)) -> Self {
        let mut base_builder = DynamicStyleBuilder::new();
        style_builder(&mut base_builder);

        DynamicStyle {
            base: base_builder.attributes,
            ..default()
        }
    }

    pub fn need_pseudo_state(&self) -> bool {
        self.enabled.is_some()
            || self.disabled.is_some()
            || self.selected.is_some()
            || self.checked.is_some()
            || self.empty.is_some()
    }

    pub fn need_flux_interaction(&self) -> bool {
        self.base.iter().any(|attr| attr.need_flux_interaction())
            || self
                .enabled
                .as_ref()
                .is_some_and(|attrs| attrs.iter().any(|attr| attr.need_flux_interaction()))
            || self
                .disabled
                .as_ref()
                .is_some_and(|attrs| attrs.iter().any(|attr| attr.need_flux_interaction()))
            || self
                .selected
                .as_ref()
                .is_some_and(|attrs| attrs.iter().any(|attr| attr.need_flux_interaction()))
            || self
                .checked
                .as_ref()
                .is_some_and(|attrs| attrs.iter().any(|attr| attr.need_flux_interaction()))
            || self
                .empty
                .as_ref()
                .is_some_and(|attrs| attrs.iter().any(|attr| attr.need_flux_interaction()))
    }

    pub fn apply<'a>(&self, state: Option<PseudoState>, entity: Entity, mut commands: Commands) {
        let attributes: &Vec<DynamicStyleAttributes> = if let Some(state) = state {
            match state {
                PseudoState::Enabled => {
                    if let Some(ref attr) = self.enabled {
                        attr
                    } else {
                        &self.base
                    }
                }
                PseudoState::Disabled => {
                    if let Some(ref attr) = self.disabled {
                        attr
                    } else {
                        &self.base
                    }
                }
                PseudoState::Selected => {
                    if let Some(ref attr) = self.selected {
                        attr
                    } else {
                        &self.base
                    }
                }
                PseudoState::Checked => {
                    if let Some(ref attr) = self.checked {
                        attr
                    } else {
                        &self.base
                    }
                }
                PseudoState::Empty => {
                    if let Some(ref attr) = self.empty {
                        attr
                    } else {
                        &self.base
                    }
                }
            }
        } else {
            &self.base
        };

        for attr in attributes.iter() {
            attr.apply(&mut commands.style(entity));
        }
    }

    pseudo_state_setter!(enabled);
    pseudo_state_setter!(disabled);
    pseudo_state_setter!(selected);
    pseudo_state_setter!(checked);
    pseudo_state_setter!(empty);
}

impl Default for Theme<FloatingPanelFoldButton> {
    fn default() -> Self {
        Self {
            context: Default::default(),
            style: DynamicStyle::build(|style_builder| {
                style_builder
                    .background_color(Color::WHITE)
                    .hover(Color::rgba(0., 1., 1., 1.))
                    .animate()
                    .all(
                        0.1,
                        Ease::OutExpo,
                        None,
                        AnimationLoop::PingPongContinous.into(),
                    );
            }),
        }
    }
}
