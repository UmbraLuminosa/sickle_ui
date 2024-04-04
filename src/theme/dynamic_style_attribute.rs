use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use sickle_math::lerp::Lerp;

use crate::{
    ui_style::{SetBackgroundColorExt, SetBorderColorExt, UiStyle},
    FluxInteraction, FluxInteractionStopwatch,
};

use super::{AnimationProgress, StyleAnimation};

/*
DynamicStyle::build(|builder| {
    builder
        .inert() // InertDynamicStyleBuilder
        .background_color(Color::BLUE)
        .border_color(Color::BLUE);
    builder
        .interactive() // InteractiveDynamicStyleBuilder
        // ideally cached as part of theming commons
        .background_color(AttributeBundle<Color>{ Color::BLUE, Color::RED.into(), None, None, None });
    builder
        .animated() // AnimatedDynamicStyleBuilder
        .background_color(LerpAttributeBundle<Color>{ Color::BLUE, Color::RED.into(), None, None, None })
        .pointer_enter(0.2, Ease::InOutExpo, None, LoopType::Continous)
        .pointer_leave(0.2, Ease::InOutExpo, Some(0.2), None);
});

DynamicAttribute:
- Inert values:
  - No update after initial setting
  - Any stylable attribute
  - Custom inert value:
    - Single callback
- Flux-static values
  - Update per flux status change
  - Any stylable attribute
  - Custom flux-static values:
    - Callback with flux state
- Animated interactive values
  - Update per frame
  - Potentially locking stopwatch
  - Any stylable Lerp attribute
  - Custom animated values:
    - Callback with flux state and animation progress / loop

pub enum DynamicAttribute {
  // Remove on apply
  Static(StaticStyleAttribute),

  // Needs flux
  Interactive(InteractiveStyleAttribute),

  // Needs stopwatch
  // None animations are effectively Pop
  // Only Lerp properties
  Animated {
    attribute: AnimatedStyleAttribute,
    // Remove Option<> around animation prop
    controller: DynamicStyleController
  }

  // apply() returns lock length (None, f32, Indefinite)
  // impl is_inert, is_interactive, is_animated
}

*/

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicStyleExtremes<T: Lerp + Default + Clone + Copy + PartialEq> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicStyleState {
    phase: FluxInteraction,
    iteration: u8,
    animation: AnimationProgress,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DynamicStyleController {
    animation: Option<StyleAnimation>,
    // Ideally only set when press starts. Otherwise everytime PointerEnter updated
    transition_base: DynamicStyleState,
    state: DynamicStyleState,
    dirty: bool,
}

impl Default for DynamicStyleController {
    fn default() -> Self {
        Self {
            animation: Default::default(),
            transition_base: Default::default(),
            state: Default::default(),
            dirty: true,
        }
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

pub struct CustomDynamicStyle {
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
    BorderColor(DynamicStyleAttribute<Color>),
    CustomF32(fn(f32, Entity, &mut World), DynamicStyleAttribute<f32>),
    // TODO: Implement an additional Custom value that also knows which flux state it is in
}

impl PartialEq for DynamicStyleAttributes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackgroundColor(_), Self::BackgroundColor(_)) => true,
            (Self::CustomF32(l0, _), Self::CustomF32(r0, _)) => l0 == r0,
            _ => false,
        }
    }
}

impl DynamicStyleAttributes {
    pub fn need_flux_interaction(&self) -> bool {
        match self {
            DynamicStyleAttributes::BackgroundColor(attr)
            | DynamicStyleAttributes::BorderColor(attr) => attr.need_flux_interaction(),
            DynamicStyleAttributes::CustomF32(_, attr) => attr.need_flux_interaction(),
        }
    }

    pub fn dirty(&self) -> bool {
        match self {
            DynamicStyleAttributes::BackgroundColor(attr)
            | DynamicStyleAttributes::BorderColor(attr) => attr.dirty,
            DynamicStyleAttributes::CustomF32(_, attr) => attr.dirty,
        }
    }

    fn set_dirty(&mut self, dirty: bool) {
        match self {
            DynamicStyleAttributes::BackgroundColor(attr)
            | DynamicStyleAttributes::BorderColor(attr) => attr.dirty = dirty,
            DynamicStyleAttributes::CustomF32(_, attr) => attr.dirty = dirty,
        }
    }

    pub fn update(
        &mut self,
        flux_interaction: &FluxInteraction,
        stopwatch: &FluxInteractionStopwatch,
    ) {
    }

    pub fn apply<'a>(&'a mut self, ui_style: &'a mut UiStyle<'a>) {
        if !self.dirty() {
            return;
        }

        match self {
            DynamicStyleAttributes::BackgroundColor(attr) => {
                ui_style.background_color(attr.current);
            }
            DynamicStyleAttributes::BorderColor(attr) => {
                ui_style.border_color(attr.current);
            }
            DynamicStyleAttributes::CustomF32(callback, attr) => {
                ui_style.entity_commands().add(CustomDynamicStyle {
                    callback: *callback,
                    current_value: attr.current,
                });
            }
        };

        self.set_dirty(false);
    }
}
