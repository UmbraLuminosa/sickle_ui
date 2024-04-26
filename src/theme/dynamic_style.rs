use bevy::{prelude::*, time::Stopwatch, ui::UiSystem};

use crate::{ui_style::UiStyleExt, FluxInteraction, StopwatchLock};

use super::*;

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
                tick_dynamic_style_stopwatch,
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

fn tick_dynamic_style_stopwatch(
    time: Res<Time<Real>>,
    mut q_stopwatches: Query<(Entity, &mut DynamicStyleStopwatch)>,
    mut commands: Commands,
) {
    for (entity, mut style_stopwatch) in &mut q_stopwatches {
        let remove_stopwatch = match style_stopwatch.1 {
            StopwatchLock::None => true,
            StopwatchLock::Infinite => false,
            StopwatchLock::Duration(length) => style_stopwatch.0.elapsed() > length,
        };

        if remove_stopwatch {
            commands.entity(entity).remove::<DynamicStyleStopwatch>();
        }

        style_stopwatch.0.tick(time.delta());
    }
}

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
            let style = style.bypass_change_detection();
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
    mut q_styles: Query<
        (
            Entity,
            &DynamicStyle,
            &FluxInteraction,
            Option<&mut DynamicStyleStopwatch>,
        ),
        Or<(Changed<DynamicStyle>, Changed<FluxInteraction>)>,
    >,
    mut commands: Commands,
) {
    for (entity, style, interaction, stopwatch) in &mut q_styles {
        let mut lock_needed = StopwatchLock::None;
        let mut keep_stop_watch = false;

        for attribute in &style.0 {
            match attribute {
                DynamicStyleAttribute::Interactive(style) => {
                    style.apply(*interaction, &mut commands.style(entity));
                }
                DynamicStyleAttribute::Animated { controller, .. } => {
                    let animation_lock = if controller.entering() {
                        keep_stop_watch = true;

                        controller.animation.lock_duration(&FluxInteraction::None)
                            + controller.animation.lock_duration(interaction)
                    } else {
                        controller.animation.lock_duration(interaction)
                    };

                    if animation_lock > lock_needed {
                        lock_needed = animation_lock;
                    }
                }
                _ => continue,
            }
        }

        if let Some(mut stopwatch) = stopwatch {
            if !keep_stop_watch {
                stopwatch.0.reset();
            }
            stopwatch.1 = lock_needed;
        } else {
            commands
                .entity(entity)
                .insert(DynamicStyleStopwatch(Stopwatch::new(), lock_needed));
        }
    }
}

fn update_dynamic_style_on_stopwatch_change(
    mut q_styles: Query<
        (
            Entity,
            &mut DynamicStyle,
            &FluxInteraction,
            Option<&DynamicStyleStopwatch>,
        ),
        Or<(
            Changed<DynamicStyle>,
            Changed<FluxInteraction>,
            Changed<DynamicStyleStopwatch>,
        )>,
    >,
    mut commands: Commands,
) {
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
                controller.update(interaction, stopwatch.0.elapsed_secs());
            }

            if style_changed || controller.dirty() {
                attribute.apply(controller.current_state(), &mut commands.style(entity));
            }
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct DynamicStyleStopwatch(pub Stopwatch, pub StopwatchLock);

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
}
