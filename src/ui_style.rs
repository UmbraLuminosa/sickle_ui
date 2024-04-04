use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
    utils::HashSet,
};
use sickle_macros::StyleCommands;

use crate::{theme::style_animation::AnimationProgress, FluxInteraction};

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl<'a> UiStyle<'a> {
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
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

    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
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

/// Derive leaves the original struct, ignore it.
/// (derive macros have a better style overall)
#[derive(StyleCommands)]
enum _StyleAttributes {
    Display {
        display: Display,
    },
    PositionType {
        position_type: PositionType,
    },
    Overflow {
        overflow: Overflow,
    },
    Direction {
        direction: Direction,
    },
    #[animatable]
    Left {
        left: Val,
    },
    #[animatable]
    Right {
        right: Val,
    },
    #[animatable]
    Top {
        top: Val,
    },
    #[animatable]
    Bottom {
        bottom: Val,
    },
    #[animatable]
    Width {
        width: Val,
    },
    #[animatable]
    Height {
        height: Val,
    },
    #[animatable]
    MinWidth {
        min_width: Val,
    },
    #[animatable]
    MinHeight {
        min_height: Val,
    },
    AspectRatio {
        aspect_ratio: Option<f32>,
    },
    AlignItems {
        align_items: AlignItems,
    },
    JustifyItems {
        justify_items: JustifyItems,
    },
    AlignSelf {
        align_self: AlignSelf,
    },
    JustifySelf {
        justify_self: JustifySelf,
    },
    AlignContent {
        align_content: AlignContent,
    },
    JustifyContents {
        justify_content: JustifyContent,
    },
    #[animatable]
    Margin {
        margin: UiRect,
    },
    #[animatable]
    Padding {
        padding: UiRect,
    },
    #[animatable]
    Border {
        border: UiRect,
    },
    FlexDirection {
        flex_direction: FlexDirection,
    },
    FlexWrap {
        flex_wrap: FlexWrap,
    },
    #[animatable]
    FlexGrow {
        flex_grow: f32,
    },
    #[animatable]
    FlexShrink {
        flex_shrink: f32,
    },
    #[animatable]
    FlexBasis {
        flex_basis: Val,
    },
    #[animatable]
    RowGap {
        row_gap: Val,
    },
    #[animatable]
    ColumnGap {
        column_gap: Val,
    },
    GridAutoFlow {
        grid_auto_flow: GridAutoFlow,
    },
    GridTemplateRows {
        grid_template_rows: Vec<RepeatedGridTrack>,
    },
    GridTemplateColumns {
        grid_template_columns: Vec<RepeatedGridTrack>,
    },
    GridAutoRows {
        grid_auto_rows: Vec<GridTrack>,
    },
    GridAutoColumns {
        grid_auto_columns: Vec<GridTrack>,
    },
    GridRow {
        grid_row: GridPlacement,
    },
    GridColumn {
        grid_column: GridPlacement,
    },
    #[target_tupl(BackgroundColor)]
    #[animatable]
    BackgroundColor {
        background_color: Color,
    },
    #[target_tupl(BorderColor)]
    BorderColor {
        border_color: Color,
    },
    #[target_enum]
    FocusPolicy {
        focus_policy: FocusPolicy,
    },
    #[target_enum]
    Visibility {
        visibility: Visibility,
    },
    #[skip_enity_command]
    ZIndex {
        z_index: ZIndex,
    },
    #[skip_ui_style_ext]
    Image {
        image: String,
    },
    #[skip_enity_command]
    ImageScaleMode {
        image_scale_mode: ImageScaleMode,
    },
    #[static_style_only]
    #[skip_ui_style_ext]
    FluxInteraction {
        flux_interaction_enabled: bool,
    },
    #[skip_styleable_enum]
    #[skip_ui_style_ext]
    AbsolutePosition {
        absolute_position: Vec2,
    },
}

#[derive(Component, Debug, Default)]
pub struct LockedStyleAttributes(HashSet<StylableAttribute>);

impl LockedStyleAttributes {
    pub fn contains(&self, attr: StylableAttribute) -> bool {
        self.0.contains(&attr)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InteractionAnimationState {
    phase: crate::FluxInteraction,
    iteration: u8,
    animation: AnimationProgress,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StaticValueBundle<T: Clone> {
    pub base: T,
    pub hover: Option<T>,
    pub press: Option<T>,
    pub cancel: Option<T>,
    // focus: Option<T>,
}

impl<T: Clone> StaticValueBundle<T> {
    fn to_value(&self, flux_interaction: crate::FluxInteraction) -> T {
        match flux_interaction {
            FluxInteraction::None => self.base.clone(),
            FluxInteraction::PointerEnter => self.hover.clone().unwrap_or(self.base.clone()),
            FluxInteraction::PointerLeave => self.base.clone(),
            FluxInteraction::Pressed => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.base.clone())),
            FluxInteraction::Released => self.hover.clone().unwrap_or(self.base.clone()),
            FluxInteraction::PressCanceled => self.cancel.clone().unwrap_or(self.base.clone()),
            FluxInteraction::Disabled => self.base.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimatedValueBundle<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> {
    pub base: T,
    pub hover: Option<T>,
    pub press: Option<T>,
    pub cancel: Option<T>,
    // focus: Option<T>,
}

impl<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> AnimatedValueBundle<T> {
    fn prev_phase(flux_interaction: FluxInteraction) -> FluxInteraction {
        match flux_interaction {
            FluxInteraction::None => FluxInteraction::PointerEnter,
            FluxInteraction::PointerEnter => FluxInteraction::None,
            FluxInteraction::PointerLeave => FluxInteraction::PointerEnter,
            FluxInteraction::Pressed => FluxInteraction::PointerEnter,
            FluxInteraction::Released => FluxInteraction::Pressed,
            FluxInteraction::PressCanceled => FluxInteraction::Pressed,
            FluxInteraction::Disabled => FluxInteraction::None,
        }
    }
    fn phase_value(&self, flux_interaction: FluxInteraction) -> T {
        match flux_interaction {
            FluxInteraction::None => self.base.clone(),
            FluxInteraction::PointerEnter => self.hover.clone().unwrap_or(self.base.clone()),
            FluxInteraction::PointerLeave => self.base.clone(),
            FluxInteraction::Pressed => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.base.clone())),
            FluxInteraction::Released => self.hover.clone().unwrap_or(self.base.clone()),
            FluxInteraction::PressCanceled => self.cancel.clone().unwrap_or(self.base.clone()),
            FluxInteraction::Disabled => self.base.clone(),
        }
    }

    fn to_value(
        &self,
        transition_base: InteractionAnimationState,
        animation_progress: InteractionAnimationState,
    ) -> T {
        let start_value = if animation_progress.phase == FluxInteraction::Pressed {
            let prev_value = self.phase_value(FluxInteraction::None);
            let hover_value = self.phase_value(FluxInteraction::PointerEnter);
            match transition_base.animation {
                AnimationProgress::Start => prev_value,
                AnimationProgress::Inbetween(t) => prev_value.lerp(hover_value, t),
                AnimationProgress::End => hover_value,
            }
        } else {
            self.phase_value(AnimatedValueBundle::<T>::prev_phase(
                animation_progress.phase,
            ))
        };

        let end_value = self.phase_value(animation_progress.phase);
        match animation_progress.animation {
            AnimationProgress::Start => start_value,
            AnimationProgress::Inbetween(t) => start_value.lerp(end_value, t),
            AnimationProgress::End => end_value,
        }
    }
}

pub struct CustomInteractiveStyleAttribute {
    callback: fn(Entity, FluxInteraction, &mut World),
    flux_interaction: FluxInteraction,
}

impl EntityCommand for CustomInteractiveStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(id, self.flux_interaction, world);
    }
}

pub struct CustomAnimatableStyleAttribute {
    callback: fn(Entity, InteractionAnimationState, InteractionAnimationState, &mut World),
    transition_base: InteractionAnimationState,
    animation_progress: InteractionAnimationState,
}

impl EntityCommand for CustomAnimatableStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(id, self.transition_base, self.animation_progress, world);
    }
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

        *scale_mode = self.image_scale_mode;
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
            .add(SetVisibility {
                visibility: Visibility::Inherited,
                check_lock: true,
            })
            .add(SetDisplay {
                display: Display::Flex,
                check_lock: true,
            });
        self
    }

    fn hide(&'a mut self) -> &mut UiStyle<'a> {
        self.commands
            .add(SetVisibility {
                visibility: Visibility::Hidden,
                check_lock: true,
            })
            .add(SetDisplay {
                display: Display::None,
                check_lock: true,
            });
        self
    }

    fn render(&'a mut self, render: bool) -> &mut UiStyle<'a> {
        if render {
            self.commands
                .add(SetVisibility {
                    visibility: Visibility::Inherited,
                    check_lock: true,
                })
                .add(SetDisplay {
                    display: Display::Flex,
                    check_lock: true,
                });
        } else {
            self.commands
                .add(SetVisibility {
                    visibility: Visibility::Hidden,
                    check_lock: true,
                })
                .add(SetDisplay {
                    display: Display::None,
                    check_lock: true,
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
            .add(SetVisibility {
                visibility: Visibility::Inherited,
                check_lock: false,
            })
            .add(SetDisplay {
                display: Display::Flex,
                check_lock: false,
            });
        self
    }

    fn hide(&'a mut self) -> &mut UiStyleUnchecked<'a> {
        self.commands
            .add(SetVisibility {
                visibility: Visibility::Hidden,
                check_lock: false,
            })
            .add(SetDisplay {
                display: Display::None,

                check_lock: false,
            });
        self
    }

    fn render(&'a mut self, render: bool) -> &mut UiStyleUnchecked<'a> {
        if render {
            self.commands
                .add(SetVisibility {
                    visibility: Visibility::Inherited,
                    check_lock: false,
                })
                .add(SetDisplay {
                    display: Display::Flex,
                    check_lock: false,
                });
        } else {
            self.commands
                .add(SetVisibility {
                    visibility: Visibility::Hidden,
                    check_lock: false,
                })
                .add(SetDisplay {
                    display: Display::None,
                    check_lock: false,
                });
        }

        self
    }
}

struct SetAbsolutePosition {
    absolute_position: Vec2,
    check_lock: bool,
}

impl EntityCommand for SetAbsolutePosition {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "position_type",
                StylableAttribute::PositionType
            );
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

        style.position_type = PositionType::Absolute;
        style.top = Val::Px(self.absolute_position.y - offset.y);
        style.left = Val::Px(self.absolute_position.x - offset.x);
    }
}

pub trait SetAbsolutePositionExt<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyle<'a>;
}

impl<'a> SetAbsolutePositionExt<'a> for UiStyle<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyle<'a> {
        self.commands.add(SetAbsolutePosition {
            absolute_position: position,
            check_lock: true,
        });
        self
    }
}

pub trait SetAbsolutePositionUncheckedExt<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetAbsolutePositionUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn absolute_position(&'a mut self, position: Vec2) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetAbsolutePosition {
            absolute_position: position,
            check_lock: false,
        });
        self
    }
}
