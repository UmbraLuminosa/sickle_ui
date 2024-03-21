use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
};
use sickle_macros::StyleCommand;

use crate::{
    theme::{LockedStyleAttributes, StylableAttribute},
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

pub struct UiStyleUnchecked<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyleUnchecked<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }
}

pub trait UiStyleUncheckedExt<'a> {
    fn style(&'a mut self, entity: Entity) -> UiStyleUnchecked<'a>;
}

impl<'a> UiStyleUncheckedExt<'a> for Commands<'_, '_> {
    fn style(&'a mut self, entity: Entity) -> UiStyleUnchecked<'a> {
        UiStyleUnchecked {
            commands: self.entity(entity),
        }
    }
}

// Simple Style attributes

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Display)]
struct SetNodeDisplay {
    display: Display,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::PositionType)]
struct SetNodePositionType {
    position_type: PositionType,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Overflow)]
struct SetNodeOverflow {
    overflow: Overflow,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Direction)]
struct SetNodeDirection {
    direction: Direction,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Left)]
struct SetNodeLeft {
    left: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Right)]
struct SetNodeRight {
    right: Val,
}
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Top)]
struct SetNodeTop {
    top: Val,
}
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Bottom)]
struct SetNodeBottom {
    bottom: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Width)]
struct SetNodeWidth {
    width: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Height)]
struct SetNodeHeight {
    height: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::MinWidth)]
struct SetNodeMinWidth {
    min_width: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::MinHeight)]
struct SetNodeMinHeight {
    min_height: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::AspectRatio)]
struct SetNodeAspectRatio {
    aspect_ratio: Option<f32>,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::AlignItems)]
struct SetNodeAlignItems {
    align_items: AlignItems,
}
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::JustifyItems)]
struct SetNodeJustifyItems {
    justify_items: JustifyItems,
}
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::AlignSelf)]
struct SetNodeAlignSelf {
    align_self: AlignSelf,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::JustifySelf)]
struct SetNodeJustifySelf {
    justify_self: JustifySelf,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::AlignContent)]
struct SetNodeAlignContent {
    align_content: AlignContent,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::JustifyContent)]
struct SetNodeJustifyContents {
    justify_content: JustifyContent,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Margin)]
struct SetNodeMargin {
    margin: UiRect,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Padding)]
struct SetNodePadding {
    padding: UiRect,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Border)]
struct SetNodeBorder {
    border: UiRect,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FlexDirection)]
struct SetNodeFlexDirection {
    flex_direction: FlexDirection,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FlexWrap)]
struct SetNodeFlexWrap {
    flex_wrap: FlexWrap,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FlexGrow)]
struct SetNodeFlexGrow {
    flex_grow: f32,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FlexShrink)]
struct SetNodeFlexShrink {
    flex_shrink: f32,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FlexBasis)]
struct SetNodeFlexBasis {
    flex_basis: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::RowGap)]
struct SetNodeRowGap {
    row_gap: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::ColumnGap)]
struct SetNodeColumnGap {
    column_gap: Val,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridAutoFlow)]
struct SetNodeGridAutoFlow {
    grid_auto_flow: GridAutoFlow,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridTemplateRows)]
struct SetNodeGridTemplateRows {
    grid_template_rows: Vec<RepeatedGridTrack>,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridTemplateColumns)]
struct SetNodeGridTemplateColumns {
    grid_template_columns: Vec<RepeatedGridTrack>,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridAutoRows)]
struct SetNodeGridAutoRows {
    grid_auto_rows: Vec<GridTrack>,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridAutoColumns)]
struct SetNodeGridAutoColumns {
    grid_auto_columns: Vec<GridTrack>,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridRow)]
struct SetNodeGridRow {
    grid_row: GridPlacement,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::GridColumn)]
struct SetNodeGridColumn {
    grid_column: GridPlacement,
}

// Tupl style-related components
// TODO: Handle interactive original value for this and any other interactive attributes
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::BackgroundColor)]
#[target_tupl(BackgroundColor)]
struct SetBackgroundColor {
    background_color: Color,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::BorderColor)]
#[target_tupl(BorderColor)]
struct SetBorderColor {
    border_color: Color,
}

#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::FocusPolicy)]
#[target_enum]
struct SetFocusPolicy {
    focus_policy: FocusPolicy,
}

// Enum style-related components
#[derive(StyleCommand)]
#[lock_attr(StylableAttribute::Visibility)]
#[target_enum]
struct SetEntityVisiblity {
    visibility: Visibility,
}

// Special style-related components needing manual implementation
macro_rules! check_lock {
    ($world:expr, $entity:expr, $prop:literal, $lock_attr:path) => {
        if let Some(locked_attrs) = $world.get::<LockedStyleAttributes>($entity) {
            if locked_attrs.contains($lock_attr) {
                warn!(
                    "Failed to style {} property on entity {:?}: Attribute locked!",
                    $prop, $entity
                );
                return;
            }
        }
    };
}

struct SetZIndex {
    z_index: ZIndex,
    check_lock: bool,
}

impl EntityCommand for SetZIndex {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "z index", StylableAttribute::ZIndex);
        }

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
        self.commands.add(SetZIndex {
            z_index,
            check_lock: true,
        });
        self
    }
}

pub trait SetZIndexUncheckedExt<'a> {
    fn z_index(&'a mut self, z_index: ZIndex) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetZIndexUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn z_index(&'a mut self, z_index: ZIndex) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetZIndex {
            z_index,
            check_lock: false,
        });
        self
    }
}

struct SetImage {
    path: String,
    check_lock: bool,
}

impl EntityCommand for SetImage {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "image", StylableAttribute::Image);
        }

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
        self.commands.add(SetImage {
            path: path.into(),
            check_lock: true,
        });
        self
    }
}

pub trait SetImageUncheckedExt<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetImageUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn image(&'a mut self, path: impl Into<String>) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetImage {
            path: path.into(),
            check_lock: false,
        });
        self
    }
}

struct SetImageScaleMode {
    scale_mode: ImageScaleMode,
    check_lock: bool,
}

impl EntityCommand for SetImageScaleMode {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "image scale mode",
                StylableAttribute::ImageScaleMode
            );
        }

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
        self.commands.add(SetImageScaleMode {
            scale_mode,
            check_lock: true,
        });
        self
    }
}

pub trait SetImageScaleModeUncheckedExt<'a> {
    fn image_scale_mode(&'a mut self, scale_mode: ImageScaleMode) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetImageScaleModeUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn image_scale_mode(&'a mut self, scale_mode: ImageScaleMode) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetImageScaleMode {
            scale_mode,
            check_lock: false,
        });
        self
    }
}

struct SetFluxInteractionEnabled {
    enabled: bool,
    check_lock: bool,
}

impl EntityCommand for SetFluxInteractionEnabled {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "flux interaction",
                StylableAttribute::FluxInteraction
            );
        }

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
        self.commands.add(SetFluxInteractionEnabled {
            enabled: false,
            check_lock: true,
        });
        self
    }

    fn enable_flux_interaction(&'a mut self) -> &mut UiStyle<'a> {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: true,
            check_lock: true,
        });
        self
    }

    fn flux_interaction_enabled(&'a mut self, enabled: bool) -> &mut UiStyle<'a> {
        self.commands.add(SetFluxInteractionEnabled {
            enabled,
            check_lock: true,
        });
        self
    }
}

pub trait SetFluxInteractionUncheckedExt<'a> {
    fn disable_flux_interaction(&'a mut self) -> &mut UiStyleUnchecked<'a>;
    fn enable_flux_interaction(&'a mut self) -> &mut UiStyleUnchecked<'a>;
    fn flux_interaction_enabled(&'a mut self, enabled: bool) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetFluxInteractionUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn disable_flux_interaction(&'a mut self) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: false,
            check_lock: false,
        });
        self
    }

    fn enable_flux_interaction(&'a mut self) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: true,
            check_lock: false,
        });
        self
    }

    fn flux_interaction_enabled(&'a mut self, enabled: bool) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetFluxInteractionEnabled {
            enabled,
            check_lock: false,
        });
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

pub trait SetNodeShowHideUncheckedExt<'a> {
    fn show(&'a mut self) -> &mut UiStyleUnchecked<'a>;
    fn hide(&'a mut self) -> &mut UiStyleUnchecked<'a>;
    fn render(&'a mut self, render: bool) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetNodeShowHideUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn show(&'a mut self) -> &mut UiStyleUnchecked<'a> {
        self.commands
            .add(SetEntityVisiblityUnchecked {
                visibility: Visibility::Inherited,
            })
            .add(SetNodeDisplayUnchecked {
                display: Display::Flex,
            });
        self
    }

    fn hide(&'a mut self) -> &mut UiStyleUnchecked<'a> {
        self.commands
            .add(SetEntityVisiblityUnchecked {
                visibility: Visibility::Hidden,
            })
            .add(SetNodeDisplayUnchecked {
                display: Display::None,
            });
        self
    }

    fn render(&'a mut self, render: bool) -> &mut UiStyleUnchecked<'a> {
        if render {
            self.commands
                .add(SetEntityVisiblityUnchecked {
                    visibility: Visibility::Inherited,
                })
                .add(SetNodeDisplayUnchecked {
                    display: Display::Flex,
                });
        } else {
            self.commands
                .add(SetEntityVisiblityUnchecked {
                    visibility: Visibility::Hidden,
                })
                .add(SetNodeDisplayUnchecked {
                    display: Display::None,
                });
        }

        self
    }
}

struct SetAbsolutePosition {
    position: Vec2,
    check_lock: bool,
}

impl EntityCommand for SetAbsolutePosition {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "position: top", StylableAttribute::Top);
            check_lock!(world, entity, "position: left", StylableAttribute::Left);
        }

        let offset = if let Some(parent) = world.get::<Parent>(entity) {
            let Some(parent_node) = world.get::<Node>(parent.get()) else {
                warn!(
                    "Failed to set position on entity {:?}: Parent has no Node component!",
                    entity
                );
                return;
            };

            let size = parent_node.size();
            let Some(parent_transform) = world.get::<GlobalTransform>(parent.get()) else {
                warn!(
                    "Failed to set position on entity {:?}: Parent has no GlobalTransform component!",
                    entity
                );
                return;
            };

            parent_transform.translation().truncate() - (size / 2.)
        } else {
            Vec2::ZERO
        };

        let Some(mut style) = world.get_mut::<Style>(entity) else {
            warn!(
                "Failed to set position on entity {:?}: No Style component found!",
                entity
            );
            return;
        };

        style.top = Val::Px(self.position.y - offset.y);
        style.left = Val::Px(self.position.x - offset.x);
    }
}

pub trait SetAbsolutePositionExt<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyle<'a>;
}

impl<'a> SetAbsolutePositionExt<'a> for UiStyle<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyle<'a> {
        self.commands.add(SetAbsolutePosition {
            position,
            check_lock: true,
        });
        self
    }
}

pub trait SetAbsolutePositionUncheckedExt<'a> {
    fn image_scale_mode(&'a mut self, position: Vec2) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetAbsolutePositionUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn image_scale_mode(&'a mut self, position: Vec2) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetAbsolutePosition {
            position,
            check_lock: false,
        });
        self
    }
}
