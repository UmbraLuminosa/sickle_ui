use bevy::{prelude::*, ui::UiSystem};

use crate::{
    ui_commands::ManageFluxInteractionStopwatchLockExt, ui_style::UiStyleExt, FluxInteraction,
    FluxInteractionStopwatch, StopwatchLock,
};

use super::*;

const DYNAMIC_STYLE_STOPWATCH_LOCK: &'static str = "DynamicStyle";

pub struct DynamicStylePlugin;

impl Plugin for DynamicStylePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            DynamicStyleUpdate
                .after(CustomThemeUpdate)
                .before(UiSystem::Layout),
        )
        .add_systems(
            PostUpdate,
            (
                update_dynamic_style_static_attributes,
                update_dynamic_style_on_flux_change,
                update_dynamic_style_on_stopwatch_change,
            )
                .chain()
                .in_set(DynamicStyleUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DynamicStyleUpdate;

fn update_dynamic_style_static_attributes(
    mut q_styles: Query<(Entity, &mut DynamicStyle), Changed<DynamicStyle>>,
    mut commands: Commands,
) {
    for (entity, mut style) in &mut q_styles {
        let mut had_static = false;
        for attribute in &style.0 {
            let DynamicStyleAttribute::Static(style) = attribute else {
                continue;
            };

            style.apply(&mut commands.style(entity));
            had_static = true;
        }

        if had_static {
            style.0 = style
                .0
                .iter()
                .filter(|attr| !attr.is_static())
                .cloned()
                .collect();

            if style.0.len() == 0 {
                commands.entity(entity).remove::<DynamicStyle>();
            }
        }
    }
}

fn update_dynamic_style_on_flux_change(
    q_styles: Query<
        (Entity, &DynamicStyle, &FluxInteraction),
        Or<(Changed<DynamicStyle>, Changed<FluxInteraction>)>,
    >,
    mut commands: Commands,
) {
    for (entity, style, interaction) in &q_styles {
        let mut lock_needed = StopwatchLock::None;

        for attribute in &style.0 {
            match attribute {
                DynamicStyleAttribute::Interactive(style) => {
                    style.apply(*interaction, &mut commands.style(entity));
                }
                DynamicStyleAttribute::Animated {
                    controller: DynamicStyleController { animation, .. },
                    ..
                } => {
                    let animation_lock = animation.lock_duration(interaction);
                    if animation_lock > lock_needed {
                        lock_needed = animation_lock;
                    }
                }
                _ => continue,
            }
        }

        if lock_needed > StopwatchLock::None {
            commands
                .entity(entity)
                .lock_stopwatch(DYNAMIC_STYLE_STOPWATCH_LOCK, lock_needed);
        } else {
            commands
                .entity(entity)
                .try_release_stopwatch_lock(DYNAMIC_STYLE_STOPWATCH_LOCK);
        }
    }
}

fn update_dynamic_style_on_stopwatch_change(
    mut q_styles: Query<
        (
            Entity,
            &mut DynamicStyle,
            &FluxInteraction,
            Option<&FluxInteractionStopwatch>,
        ),
        Or<(
            Changed<DynamicStyle>,
            Changed<FluxInteraction>,
            Changed<FluxInteractionStopwatch>,
        )>,
    >,
    mut commands: Commands,
) {
    // TODO: Looped animation should use their own stopwatch
    // TODO: Add reset flag to times loops
    for (entity, mut style, interaction, stopwatch) in &mut q_styles {
        let style_changed = style.is_changed();
        let style = style.bypass_change_detection();

        for style_attribute in &mut style.0 {
            let DynamicStyleAttribute::Animated {
                attribute,
                ref mut controller,
            } = style_attribute
            else {
                continue;
            };

            if let Some(stopwatch) = stopwatch {
                controller.update(interaction, stopwatch);
            }

            if style_changed || controller.dirty() {
                attribute.apply(controller.current_state(), &mut commands.style(entity));
            }
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct DynamicStyle(Vec<DynamicStyleAttribute>);

impl DynamicStyle {
    pub fn new(list: Vec<DynamicStyleAttribute>) -> Self {
        Self(list)
    }

    pub fn merge(self, other: DynamicStyle) -> Self {
        let mut new_list = self.0;

        for attribute in other.0 {
            if !new_list.contains(&attribute) {
                new_list.push(attribute);
            } else {
                // Safe unwrap: checked in if above
                let index = new_list.iter().position(|dsa| *dsa == attribute).unwrap();
                new_list[index] = attribute;
            }
        }

        DynamicStyle(new_list)
    }

    pub fn copy_controllers(&mut self, other: &DynamicStyle) {
        for attribute in self.0.iter_mut() {
            if !attribute.is_animated() {
                continue;
            }

            let Some(old_attribute) = other
                .0
                .iter()
                .filter(|other_attr| other_attr.is_animated())
                .find(|other_attr| **other_attr == attribute.clone())
            else {
                continue;
            };

            let Ok(controller) = attribute.controller_mut() else {
                continue;
            };

            // Safe unwrap: attribute type already checked ^^
            let old_controller = old_attribute.controller().unwrap();
            controller.copy_state_from(old_controller);
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.0.iter().any(|attr| attr.is_interactive())
    }

    pub fn is_animated(&self) -> bool {
        self.0.iter().any(|attr| attr.is_animated())
    }

    pub fn update<'a>(
        &mut self,
        interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
        for attribute in self.0.iter_mut() {
            attribute.update(interaction, stopwatch);
        }
    }
}
