use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
    utils::HashSet,
};
use sickle_macros::StyleCommands;
use sickle_math::lerp::Lerp;

use crate::{
    theme::{
        dynamic_style::DynamicStyle,
        dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController},
        style_animation::{AnimationState, InteractionStyle, StyleAnimation},
    },
    FluxInteraction,
};

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
    #[animatable]
    MaxWidth {
        max_width: Val,
    },
    #[animatable]
    MaxHeight {
        max_height: Val,
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
    #[animatable]
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
        image_scale_mode: Option<ImageScaleMode>,
    },
    #[static_style_only]
    #[skip_ui_style_ext]
    FluxInteraction {
        flux_interaction_enabled: bool,
    },
    #[skip_lockable_enum]
    #[skip_ui_style_ext]
    AbsolutePosition {
        absolute_position: Vec2,
    },
    // TODO: Add FluxStopwatchLocked((None, f32, Indefinite) -> lockable
}

#[derive(Component, Debug, Default)]
pub struct LockedStyleAttributes(HashSet<LockableStyleAttribute>);

impl LockedStyleAttributes {
    pub fn contains(&self, attr: LockableStyleAttribute) -> bool {
        self.0.contains(&attr)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StaticBundle<T: Clone + Default> {
    pub base: T,
    pub hover: Option<T>,
    pub press: Option<T>,
    pub cancel: Option<T>,
}

impl<T: Default + Clone> From<T> for StaticBundle<T> {
    fn from(value: T) -> Self {
        StaticBundle::new(value)
    }
}

impl<T: Clone + Default> StaticBundle<T> {
    pub fn new(value: T) -> Self {
        StaticBundle {
            base: value,
            ..default()
        }
    }

    pub fn hover(self, value: T) -> Self {
        Self {
            hover: value.into(),
            ..self
        }
    }

    pub fn press(self, value: T) -> Self {
        Self {
            press: value.into(),
            ..self
        }
    }

    pub fn cancel(self, value: T) -> Self {
        Self {
            cancel: value.into(),
            ..self
        }
    }

    fn to_value(&self, flux_interaction: FluxInteraction) -> T {
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
pub struct AnimatedBundle<T: Lerp + Default + Clone + Copy + PartialEq> {
    pub base: T,
    pub hover: Option<T>,
    pub press: Option<T>,
    pub cancel: Option<T>,
}

impl<T: Lerp + Default + Clone + Copy + PartialEq> From<T> for AnimatedBundle<T> {
    fn from(value: T) -> Self {
        AnimatedBundle::new(value)
    }
}

impl<T: Lerp + Default + Clone + Copy + PartialEq> From<StaticBundle<T>> for AnimatedBundle<T> {
    fn from(value: StaticBundle<T>) -> Self {
        Self {
            base: value.base,
            hover: value.hover,
            press: value.press,
            cancel: value.cancel,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + Copy + PartialEq> AnimatedBundle<T> {
    pub fn new(value: T) -> Self {
        AnimatedBundle {
            base: value,
            ..default()
        }
    }

    pub fn hover(self, value: T) -> Self {
        Self {
            hover: value.into(),
            ..self
        }
    }

    pub fn press(self, value: T) -> Self {
        Self {
            press: value.into(),
            ..self
        }
    }

    pub fn cancel(self, value: T) -> Self {
        Self {
            cancel: value.into(),
            ..self
        }
    }

    pub fn interaction_style(&self, interaction: InteractionStyle) -> T {
        match interaction {
            InteractionStyle::Base => self.base.clone(),
            InteractionStyle::Hover => self.hover.clone().unwrap_or(self.base.clone()),
            InteractionStyle::Press => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.base.clone())),
            InteractionStyle::Cancel => self.cancel.clone().unwrap_or(self.base.clone()),
        }
    }

    fn to_value(&self, current_state: &AnimationState) -> T {
        current_state.extract(&self)
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
    callback: fn(Entity, AnimationState, &mut World),
    current_state: AnimationState,
}

impl EntityCommand for CustomAnimatableStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback)(id, self.current_state, world);
    }
}

pub struct InteractiveStyleBuilder<'a> {
    style_builder: &'a mut StyleBuilder,
}

pub struct AnimatedStyleBuilder<'a> {
    style_builder: &'a mut StyleBuilder,
}

impl<'a> AnimatedStyleBuilder<'a> {
    fn add_and_extract_animation(
        &'a mut self,
        attribute: DynamicStyleAttribute,
    ) -> &'a mut StyleAnimation {
        self.style_builder.add(attribute.clone());

        // Safe unwrap: we just added the entry, they are variant-equal
        let index = self
            .style_builder
            .attributes
            .iter()
            .position(|attr| *attr == attribute)
            .unwrap();

        let DynamicStyleAttribute::Animated {
            controller: DynamicStyleController {
                ref mut animation, ..
            },
            ..
        } = self.style_builder.attributes[index]
        else {
            unreachable!();
        };

        animation
    }

    pub fn custom(
        &'a mut self,
        callback: impl Into<fn(Entity, AnimationState, &mut World)>,
    ) -> &'a mut StyleAnimation {
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::Custom(callback.into()),
            controller: DynamicStyleController::default(),
        };

        self.add_and_extract_animation(attribute)
    }
}

pub struct StyleBuilder {
    attributes: Vec<DynamicStyleAttribute>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self { attributes: vec![] }
    }
    pub fn add(&mut self, attribute: DynamicStyleAttribute) {
        if !self.attributes.contains(&attribute) {
            self.attributes.push(attribute);
        } else {
            // Safe unwrap: checked in if above
            let index = self
                .attributes
                .iter()
                .position(|dsa| *dsa == attribute)
                .unwrap();

            warn!(
                "Overwriting {:?} with {:?}",
                self.attributes[index], attribute
            );
            self.attributes[index] = attribute;
        }
    }

    pub fn interactive<'a>(&'a mut self) -> InteractiveStyleBuilder<'a> {
        InteractiveStyleBuilder {
            style_builder: self,
        }
    }

    pub fn animated<'a>(&'a mut self) -> AnimatedStyleBuilder<'a> {
        AnimatedStyleBuilder {
            style_builder: self,
        }
    }
}

impl From<StyleBuilder> for DynamicStyle {
    fn from(value: StyleBuilder) -> Self {
        DynamicStyle::new(value.attributes)
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
            check_lock!(world, entity, "z index", LockableStyleAttribute::ZIndex);
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
            check_lock!(world, entity, "image", LockableStyleAttribute::Image);
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
                LockableStyleAttribute::ImageScaleMode
            );
        }

        if let Some(image_scale_mode) = self.image_scale_mode {
            if let Some(mut scale_mode) = world.get_mut::<ImageScaleMode>(entity) {
                *scale_mode = image_scale_mode;
            } else {
                world.entity_mut(entity).insert(image_scale_mode);
            }
        } else if let Some(_) = world.get::<ImageScaleMode>(entity) {
            world.entity_mut(entity).remove::<ImageScaleMode>();
        }
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
                LockableStyleAttribute::FluxInteraction
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
                LockableStyleAttribute::PositionType
            );
            check_lock!(world, entity, "position: top", LockableStyleAttribute::Top);
            check_lock!(
                world,
                entity,
                "position: left",
                LockableStyleAttribute::Left
            );
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
