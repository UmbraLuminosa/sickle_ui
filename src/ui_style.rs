use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
};
use sickle_macros::StyleCommand;

use crate::{
    interactions::{InteractionState, InteractiveBackgroundState},
    FluxInteraction,
};

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyle<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }
}

pub trait UiStyleExt<'a> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a>;
}

impl<'a> UiStyleExt<'a> for Commands<'_, '_> {
    fn style(&'a mut self, entity: Entity) -> UiStyle<'a> {
        UiStyle {
            commands: self.entity(entity),
        }
    }
}

#[derive(StyleCommand)]
struct SetNodePositionType {
    position_type: PositionType,
}

#[derive(StyleCommand)]
struct SetNodeWidth {
    width: Val,
}

#[derive(StyleCommand)]
struct SetNodeHeight {
    height: Val,
}

#[derive(StyleCommand)]
struct SetNodeMinWidth {
    min_width: Val,
}

#[derive(StyleCommand)]
struct SetNodeMinHeight {
    min_height: Val,
}

#[derive(StyleCommand)]
struct SetNodeTop {
    top: Val,
}

#[derive(StyleCommand)]
struct SetNodeRight {
    right: Val,
}

#[derive(StyleCommand)]
struct SetNodeBottom {
    bottom: Val,
}

#[derive(StyleCommand)]
struct SetNodeLeft {
    left: Val,
}

#[derive(StyleCommand)]
struct SetNodeOverflow {
    overflow: Overflow,
}

#[derive(StyleCommand)]
struct SetNodePadding {
    padding: UiRect,
}

#[derive(StyleCommand)]
struct SetNodeMargin {
    margin: UiRect,
}

#[derive(StyleCommand)]
struct SetNodeBorder {
    border: UiRect,
}

#[derive(StyleCommand)]
struct SetNodeFlexDirection {
    flex_direction: FlexDirection,
}

#[derive(StyleCommand)]
struct SetNodeFlexWrap {
    flex_wrap: FlexWrap,
}

#[derive(StyleCommand)]
struct SetNodeFlexGrow {
    flex_grow: f32,
}

#[derive(StyleCommand)]
struct SetNodeDisplay {
    display: Display,
}

#[derive(StyleCommand)]
struct SetNodeAlignSelf {
    align_self: AlignSelf,
}

#[derive(StyleCommand)]
struct SetNodeAlignItems {
    align_items: AlignItems,
}

#[derive(StyleCommand)]
struct SetNodeAlignContent {
    align_content: AlignContent,
}

#[derive(StyleCommand)]
struct SetNodeJustifySelf {
    justify_self: JustifySelf,
}

#[derive(StyleCommand)]
struct SetNodeJustifyItems {
    justify_items: JustifyItems,
}

#[derive(StyleCommand)]
struct SetNodeJustifyContents {
    justify_content: JustifyContent,
}

struct SetImage {
    path: String,
}

impl EntityCommand for SetImage {
    fn apply(self, entity: Entity, world: &mut World) {
        let handle = if self.path == "" {
            Handle::default()
        } else {
            world.resource::<AssetServer>().load(self.path)
        };

        let Some(mut image) = world.get_mut::<UiImage>(entity) else {
            warn!(
                "Failed to set image on entity {:?}: No UiImage component found!",
                entity
            );
            return;
        };

        if image.texture != handle {
            image.texture = handle;
        }
    }
}

pub trait SetImageExt<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'a>;
}

impl<'a> SetImageExt<'a> for UiStyle<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyle<'a> {
        self.commands.add(SetImage { path: path.into() });
        self
    }
}

struct SetImageScaleMode {
    scale_mode: ImageScaleMode,
}

impl EntityCommand for SetImageScaleMode {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut scale_mode) = world.get_mut::<ImageScaleMode>(entity) else {
            warn!(
                "Failed to set image scale mode on entity {:?}: No ImageScaleMode component found!",
                entity
            );
            return;
        };

        *scale_mode = self.scale_mode;
    }
}

pub trait SetImageScaleModeExt<'a> {
    fn image_scale_mode(&'a mut self, scale_mode: ImageScaleMode) -> &mut UiStyle<'a>;
}

impl<'a> SetImageScaleModeExt<'a> for UiStyle<'a> {
    fn image_scale_mode(&'a mut self, scale_mode: ImageScaleMode) -> &mut UiStyle<'a> {
        self.commands.add(SetImageScaleMode { scale_mode });
        self
    }
}

struct SetEntityVisiblity {
    visibility: Visibility,
}

impl EntityCommand for SetEntityVisiblity {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut visiblity) = world.get_mut::<Visibility>(entity) else {
            warn!(
                "Failed to set visiblity on entity {:?}: No Visibility component found!",
                entity
            );
            return;
        };

        if *visiblity != self.visibility {
            *visiblity = self.visibility;
        }
    }
}

pub trait SetEntityVisiblityExt<'a> {
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'a>;
}

impl<'a> SetEntityVisiblityExt<'a> for UiStyle<'a> {
    fn visibility(&'a mut self, visibility: Visibility) -> &mut UiStyle<'a> {
        self.commands.add(SetEntityVisiblity { visibility });
        self
    }
}

pub trait SetNodeShowHideExt<'a> {
    fn show(&'a mut self) -> &mut UiStyle<'a>;
    fn hide(&'a mut self) -> &mut UiStyle<'a>;
    fn render(&'a mut self, render: bool) -> &mut UiStyle<'a>;
}

impl<'a> SetNodeShowHideExt<'a> for UiStyle<'a> {
    fn show(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetEntityVisiblity {
                visibility: Visibility::Inherited,
            })
            .add(SetNodeDisplay {
                display: Display::Flex,
            });
        self
    }

    fn hide(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetEntityVisiblity {
                visibility: Visibility::Hidden,
            })
            .add(SetNodeDisplay {
                display: Display::None,
            });
        self
    }

    fn render(&'a mut self, render: bool) -> &mut UiStyle<'a> {
        if render {
            self.commands
                .add(SetEntityVisiblity {
                    visibility: Visibility::Inherited,
                })
                .add(SetNodeDisplay {
                    display: Display::Flex,
                });
        } else {
            self.commands
                .add(SetEntityVisiblity {
                    visibility: Visibility::Hidden,
                })
                .add(SetNodeDisplay {
                    display: Display::None,
                });
        }

        self
    }
}

struct SetBorderColor {
    color: Color,
}

impl EntityCommand for SetBorderColor {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut border_color) = world.get_mut::<BorderColor>(entity) else {
            warn!(
                "Failed to set border color on entity {:?}: No BorderColor component found!",
                entity
            );
            return;
        };

        if border_color.0 != self.color.into() {
            border_color.0 = self.color.into();
        }
    }
}

pub trait SetBorderColorExt<'a> {
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'a>;
}

impl<'a> SetBorderColorExt<'a> for UiStyle<'a> {
    fn border_color(&'a mut self, color: Color) -> &mut UiStyle<'a> {
        self.commands.add(SetBorderColor { color });
        self
    }
}

struct SetBackgroundColor {
    color: Color,
}

impl EntityCommand for SetBackgroundColor {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut background_color) = world.get_mut::<BackgroundColor>(entity) else {
            warn!(
                "Failed to set background color on entity {:?}: No BackgroundColor component found!",
                entity
            );
            return;
        };

        if background_color.0 != self.color.into() {
            background_color.0 = self.color.into();
        }

        // TODO: Make this CFG optional
        let Some(mut interactive_state) = world.get_mut::<InteractiveBackgroundState>(entity) else {
            return;
        };

        interactive_state.set_original(self.color);
    }
}

pub trait SetBackgroundColorExt<'a> {
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'a>;
}

impl<'a> SetBackgroundColorExt<'a> for UiStyle<'a> {
    fn background_color(&'a mut self, color: Color) -> &mut UiStyle<'a> {
        self.commands.add(SetBackgroundColor { color });
        self
    }
}

struct SetZIndex {
    z_index: ZIndex,
}

impl EntityCommand for SetZIndex {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut z_index) = world.get_mut::<ZIndex>(entity) else {
            warn!(
                "Failed to set z index on entity {:?}: No ZIndex component found!",
                entity
            );
            return;
        };

        // Best effort avoid change triggering
        if let (ZIndex::Local(level), ZIndex::Local(target)) = (*z_index, self.z_index) {
            if level != target {
                *z_index = self.z_index;
            }
        } else if let (ZIndex::Global(level), ZIndex::Global(target)) = (*z_index, self.z_index) {
            if level != target {
                *z_index = self.z_index;
            }
        } else {
            *z_index = self.z_index;
        }
    }
}

pub trait SetZIndexExt<'a> {
    fn z_index(&'a mut self, z_index: ZIndex) -> &mut UiStyle<'a>;
}

impl<'a> SetZIndexExt<'a> for UiStyle<'a> {
    fn z_index(&'a mut self, z_index: ZIndex) -> &mut UiStyle<'a> {
        self.commands.add(SetZIndex { z_index });
        self
    }
}

struct SetFocusPolicy {
    focus_policy: FocusPolicy,
}

impl EntityCommand for SetFocusPolicy {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut focus_policy) = world.get_mut::<FocusPolicy>(entity) else {
            warn!(
                "Failed to set focus policy on entity {:?}: No FocusPolicy component found!",
                entity
            );
            return;
        };

        if *focus_policy != self.focus_policy {
            *focus_policy = self.focus_policy;
        }
    }
}

pub trait SetFocusPolicyExt<'a> {
    fn focus_policy(&'a mut self, focus_policy: FocusPolicy) -> &mut UiStyle<'a>;
}

impl<'a> SetFocusPolicyExt<'a> for UiStyle<'a> {
    fn focus_policy(&'a mut self, focus_policy: FocusPolicy) -> &mut UiStyle<'a> {
        self.commands.add(SetFocusPolicy { focus_policy });
        self
    }
}

struct SetFluxInteractionEnabled {
    enabled: bool,
}

impl EntityCommand for SetFluxInteractionEnabled {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut flux_interaction) = world.get_mut::<FluxInteraction>(entity) else {
            warn!(
                "Failed to set flux interaction on entity {:?}: No FluxInteraction component found!",
                entity
            );
            return;
        };

        if self.enabled {
            if *flux_interaction == FluxInteraction::Disabled {
                *flux_interaction = FluxInteraction::None;
            }
        } else {
            if *flux_interaction != FluxInteraction::Disabled {
                *flux_interaction = FluxInteraction::Disabled;
            }
        }
    }
}

pub trait SetFluxInteractionExt<'a> {
    fn disable_flux_interaction(&'a mut self) -> &mut UiStyle<'a>;
    fn enable_flux_interaction(&'a mut self) -> &mut UiStyle<'a>;
    fn flux_interaction_enabled(&'a mut self, enabled: bool) -> &mut UiStyle<'a>;
}

impl<'a> SetFluxInteractionExt<'a> for UiStyle<'a> {
    fn disable_flux_interaction(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetFluxInteractionEnabled { enabled: false });
        self
    }

    fn enable_flux_interaction(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetFluxInteractionEnabled { enabled: true });
        self
    }

    fn flux_interaction_enabled(&'a mut self, enabled: bool) -> &mut UiStyle<'a> {
        self.commands.add(SetFluxInteractionEnabled { enabled });
        self
    }
}
