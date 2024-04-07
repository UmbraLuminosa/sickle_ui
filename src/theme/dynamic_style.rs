use bevy::prelude::*;
use sickle_math::lerp::Lerp;

use crate::{FluxInteraction, FluxInteractionStopwatch};

use super::*;

pub struct DynamicStylePlugin;

impl Plugin for DynamicStylePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DynamicStyleUpdate.after(ThemeUpdate))
            .add_systems(
                Update,
                (update_inert_dynamic_styles,).in_set(DynamicStyleUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DynamicStyleUpdate;

// These are dynamic styles with only inert values
fn update_inert_dynamic_styles(
    mut q_styles: Query<
        (Entity, &mut DynamicStyle),
        (Without<FluxDynamicStyle>, Changed<DynamicStyle>),
    >,
    mut commands: Commands,
) {
    for (entity, mut style) in &mut q_styles {
        style.apply(entity, commands.reborrow());
    }
}

fn update_flux_dynamic_styles(
    mut q_styles: Query<
        (
            Entity,
            &mut DynamicStyle,
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
    for (entity, mut style, _interaction, _stopwatch) in &mut q_styles {
        style.apply(entity, commands.reborrow());
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FluxDynamicStyle;

#[derive(Component, Clone, Debug)]
pub struct DynamicStyle(Vec<DynamicStyleAttributes>);

impl DynamicStyle {
    pub fn merge(self, other: DynamicStyle) -> Self {
        let mut new_list = other.0;
        for attr in self.0 {
            if !new_list.contains(&attr) {
                new_list.push(attr);
            }
        }

        DynamicStyle(new_list)
    }

    pub fn build(style_builder: impl FnOnce(&mut DynamicStyleBuilder)) -> Self {
        let mut base_builder = DynamicStyleBuilder::new();
        style_builder(&mut base_builder);

        DynamicStyle(base_builder.attributes)
    }

    pub fn need_flux_interaction(&self) -> bool {
        self.0.iter().any(|attr| attr.need_flux_interaction())
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

    pub fn apply(&mut self, entity: Entity, mut commands: Commands) {
        for attr in self.0.iter_mut() {
            attr.apply(&mut commands.style(entity));
        }
    }
}

pub struct DynamicStyleBuilder {
    attributes: Vec<DynamicStyleAttributes>,
}

macro_rules! style_builder {
    ($func_name:ident, $variant:path, $type:ident) => {
        pub fn $func_name(&mut self, base: $type) -> &mut DynamicStyleAttribute<$type> {
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

    pub fn custom(
        &mut self,
        base: f32,
        callback: fn(f32, Entity, &mut World),
    ) -> &mut DynamicStyleAttribute<f32> {
        let attribute = DynamicStyleAttribute::new(base);
        let variant = DynamicStyleAttributes::CustomF32(callback, attribute);

        if self.attributes.contains(&variant) {
            let Some(&mut DynamicStyleAttributes::CustomF32(_, ref mut unwrapped_attr)) =
                self.attributes.iter_mut().find(|item| **item == variant)
            else {
                unreachable!();
            };

            unwrapped_attr.base(base);
            return unwrapped_attr;
        }

        self.attributes.push(variant);

        let Some(&mut DynamicStyleAttributes::CustomF32(_, ref mut unwrapped_attr)) =
            self.attributes.last_mut()
        else {
            unreachable!();
        };

        unwrapped_attr
    }
}
