use bevy::prelude::*;

use crate::{ui_style::UiStyleExt, FluxInteraction, FluxInteractionStopwatch};

use super::*;

pub struct DynamicStylePlugin;

impl Plugin for DynamicStylePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DynamicStyleUpdate.after(ThemeUpdate))
            .add_systems(
                Update,
                (
                    update_dynamic_style_static_attributes,
                    update_dynamic_style_interactive_attributes,
                    update_dynamic_style_animated_attributes,
                )
                    .chain()
                    .in_set(DynamicStyleUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DynamicStyleUpdate;

// These are dynamic styles with only inert values
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

fn update_dynamic_style_interactive_attributes(
    q_styles: Query<
        (Entity, &DynamicStyle, &FluxInteraction),
        Or<(Changed<DynamicStyle>, Changed<FluxInteraction>)>,
    >,
    mut commands: Commands,
) {
    for (entity, style, interaction) in &q_styles {
        for attribute in &style.0 {
            let DynamicStyleAttribute::Interactive(style) = attribute else {
                continue;
            };

            style.apply(*interaction, &mut commands.style(entity));
        }
    }
}

fn update_dynamic_style_animated_attributes(
    mut q_styles: Query<
        (
            Entity,
            &mut DynamicStyle,
            &FluxInteraction,
            &FluxInteractionStopwatch,
        ),
        Or<(
            Changed<DynamicStyle>,
            Changed<FluxInteraction>,
            Changed<FluxInteractionStopwatch>,
        )>,
    >,
    mut commands: Commands,
) {
    for (entity, mut style, interaction, stopwatch) in &mut q_styles {
        for attribute in &mut style.0 {
            let DynamicStyleAttribute::Animated {
                attribute,
                ref mut controller,
            } = attribute
            else {
                continue;
            };

            // TODO: Update stopwatch lock if interaction is_changed

            controller.update(interaction, stopwatch);
            if controller.dirty() {
                attribute.apply(
                    controller.transition_base(),
                    controller.current_state(),
                    &mut commands.style(entity),
                );
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
        let mut new_list = other.0;
        for attr in self.0 {
            if !new_list.contains(&attr) {
                new_list.push(attr);
            }
        }

        DynamicStyle(new_list)
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
        for attr in self.0.iter_mut() {
            attr.update(interaction, stopwatch);
        }
    }
}
