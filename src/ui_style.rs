use bevy::{
    ecs::system::{EntityCommand, EntityCommands},
    prelude::*,
    text::TextLayoutInfo,
    ui::{widget::TextFlags, FocusPolicy},
    utils::HashSet,
};
use serde::{Deserialize, Serialize};
use sickle_macros::StyleCommands;
use sickle_math::lerp::Lerp;

use crate::{
    theme::{
        dynamic_style::{ContextStyleAttribute, DynamicStyle},
        dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController},
        icons::IconData,
        style_animation::{AnimationSettings, AnimationState, InteractionStyle},
        typography::SizedFont,
        UiContext,
    },
    FluxInteraction,
};

use std::sync::Arc;

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

pub trait LogicalEq<Rhs: ?Sized = Self> {
    fn logical_eq(&self, other: &Rhs) -> bool;

    fn logical_ne(&self, other: &Rhs) -> bool {
        !self.logical_eq(other)
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
    #[animatable]
    ImageTint {
        image_tint: Color,
    },
    #[skip_enity_command]
    ImageFlip {
        image_flip: BVec2,
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
    #[skip_lockable_enum]
    #[skip_enity_command]
    Icon {
        icon: IconData,
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    Font {
        font: String,
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    #[animatable]
    FontSize {
        font_size: f32,
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    SizedFont {
        sized_font: SizedFont,
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    #[animatable]
    FontColor {
        font_color: Color,
    },
    #[skip_enity_command]
    #[animatable]
    Scale {
        scale: f32,
    },
}

#[derive(Component, Debug, Default, Reflect)]
pub struct LockedStyleAttributes(HashSet<LockableStyleAttribute>);

impl LockedStyleAttributes {
    pub fn new(attributes: impl Into<HashSet<LockableStyleAttribute>>) -> Self {
        Self(attributes.into())
    }

    pub fn contains(&self, attr: LockableStyleAttribute) -> bool {
        self.0.contains(&attr)
    }
}

impl From<LockableStyleAttribute> for HashSet<LockableStyleAttribute> {
    fn from(value: LockableStyleAttribute) -> Self {
        let mut set = HashSet::<LockableStyleAttribute>::new();
        set.insert(value);
        set
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct InteractiveVals<T: Clone + Default> {
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
}

impl<T: Default + Clone> From<T> for InteractiveVals<T> {
    fn from(value: T) -> Self {
        InteractiveVals::new(value)
    }
}

impl<T: Default + Clone + PartialEq> PartialEq for InteractiveVals<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idle == other.idle
            && self.hover == other.hover
            && self.press == other.press
            && self.cancel == other.cancel
    }
}

impl<T: Clone + Default> InteractiveVals<T> {
    pub fn new(value: T) -> Self {
        InteractiveVals {
            idle: value,
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

    pub fn to_value(&self, flux_interaction: FluxInteraction) -> T {
        match flux_interaction {
            FluxInteraction::None => self.idle.clone(),
            FluxInteraction::PointerEnter => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PointerLeave => self.idle.clone(),
            FluxInteraction::Pressed => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            FluxInteraction::Released => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PressCanceled => self.cancel.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::Disabled => self.idle.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimatedVals<T: Lerp + Default + Clone + PartialEq> {
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
    #[reflect(default)]
    pub idle_alt: Option<T>,
    #[reflect(default)]
    pub hover_alt: Option<T>,
    #[reflect(default)]
    pub press_alt: Option<T>,
    #[reflect(default)]
    pub enter_from: Option<T>,
}

impl<T: Lerp + Default + Clone + PartialEq> From<T> for AnimatedVals<T> {
    fn from(value: T) -> Self {
        AnimatedVals {
            idle: value,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> From<InteractiveVals<T>> for AnimatedVals<T> {
    fn from(value: InteractiveVals<T>) -> Self {
        Self {
            idle: value.idle,
            hover: value.hover,
            press: value.press,
            cancel: value.cancel,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> AnimatedVals<T> {
    pub fn interaction_style(&self, interaction: InteractionStyle) -> T {
        match interaction {
            InteractionStyle::Idle => self.idle.clone(),
            InteractionStyle::Hover => self.hover.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::Press => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Cancel => self.cancel.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::IdleAlt => self
                .idle_alt
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::HoverAlt => self.hover_alt.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::PressAlt => self
                .press_alt
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Enter => self.enter_from.clone().unwrap_or(self.idle.clone()),
        }
    }

    pub fn to_value(&self, current_state: &AnimationState) -> T {
        current_state.extract(&self)
    }
}

#[derive(Clone)]
pub struct CustomStaticStyleAttribute {
    callback: Arc<dyn Fn(Entity, &mut World) + Send + Sync + 'static>,
}

impl CustomStaticStyleAttribute {
    pub fn new(callback: impl Fn(Entity, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl std::fmt::Debug for CustomStaticStyleAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomStaticStyleAttribute").finish()
    }
}

impl PartialEq for CustomStaticStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct CustomInteractiveStyleAttribute {
    callback: Arc<dyn Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static>,
}

impl CustomInteractiveStyleAttribute {
    pub fn new(
        callback: impl Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl std::fmt::Debug for CustomInteractiveStyleAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomInteractiveStyleAttribute").finish()
    }
}

impl PartialEq for CustomInteractiveStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct CustomAnimatedStyleAttribute {
    callback: Arc<dyn Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static>,
}

impl CustomAnimatedStyleAttribute {
    pub fn new(
        callback: impl Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }
}

impl std::fmt::Debug for CustomAnimatedStyleAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomAnimatedStyleAttribute").finish()
    }
}

impl PartialEq for CustomAnimatedStyleAttribute {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

pub struct ApplyCustomStaticStyleAttribute {
    callback: CustomStaticStyleAttribute,
}

impl EntityCommand for ApplyCustomStaticStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, world);
    }
}

pub struct ApplyCustomInteractiveStyleAttribute {
    callback: CustomInteractiveStyleAttribute,
    flux_interaction: FluxInteraction,
}

impl EntityCommand for ApplyCustomInteractiveStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, self.flux_interaction, world);
    }
}

pub struct ApplyCustomAnimatadStyleAttribute {
    callback: CustomAnimatedStyleAttribute,
    current_state: AnimationState,
}

impl EntityCommand for ApplyCustomAnimatadStyleAttribute {
    fn apply(self, id: Entity, world: &mut World) {
        (self.callback.callback)(id, self.current_state, world);
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
    ) -> &'a mut AnimationSettings {
        let index = self.style_builder.add(attribute.clone());

        let DynamicStyleAttribute::Animated {
            controller: DynamicStyleController {
                ref mut animation, ..
            },
            ..
        } = self.style_builder.attributes[index].1
        else {
            unreachable!();
        };

        animation
    }

    pub fn custom(
        &'a mut self,
        callback: impl Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static,
    ) -> &'a mut AnimationSettings {
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::Custom(CustomAnimatedStyleAttribute::new(callback)),
            controller: DynamicStyleController::default(),
        };

        self.add_and_extract_animation(attribute)
    }
}

pub struct StyleBuilder {
    context: Option<String>,
    attributes: Vec<(Option<String>, DynamicStyleAttribute)>,
}

impl From<StyleBuilder> for DynamicStyle {
    fn from(value: StyleBuilder) -> Self {
        value
            .attributes
            .iter()
            .for_each(|attr| if let Some(context) = &attr.0 {
                warn!("StyleBuilder with context-bound attributes converted without context! [{}] attributes discarded!", context);
            });

        DynamicStyle::new(
            value
                .attributes
                .iter()
                .filter(|attr| attr.0.is_none())
                .map(|attr| attr.1.clone())
                .collect(),
        )
    }
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            context: None,
            attributes: vec![],
        }
    }

    pub fn add(&mut self, attribute: DynamicStyleAttribute) -> usize {
        if !self
            .attributes
            .iter()
            .any(|dsa| dsa.0 == self.context && dsa.1.logical_eq(&attribute))
        {
            self.attributes.push((self.context.clone(), attribute));
            self.attributes.len() - 1
        } else {
            // Safe unwrap: checked in if above
            let index = self
                .attributes
                .iter()
                .position(|dsa| dsa.0 == self.context && dsa.1.logical_eq(&attribute))
                .unwrap();

            warn!(
                "Overwriting {:?} with {:?}",
                self.attributes[index], attribute
            );
            self.attributes[index] = (self.context.clone(), attribute);

            index
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

    /// Revert StyleBuilder to set style on the main entity.
    pub fn reset_context(&mut self) -> &mut Self {
        self.context = None;
        self
    }

    /// All subsequent calls to the StyleBuilder will add styling to the selected sub-component.
    /// NOTE: The DynamicStyle will still be set on the main entity and interactions will be
    /// detected on it. This allows styling sub-components by the main widget.
    pub fn switch_context(&mut self, context: &'static str) -> &mut Self {
        self.context = Some(context.into());
        self
    }

    pub fn convert_with(self, context: &impl UiContext) -> DynamicStyle {
        let mut attributes: Vec<ContextStyleAttribute> = Vec::with_capacity(self.attributes.len());
        for attribute in self.attributes {
            let new_entry: ContextStyleAttribute = match attribute.0 {
                Some(target) => match context.get(target.as_str()) {
                    Ok(target_entity) => {
                        ContextStyleAttribute::new(target_entity, attribute.1.clone()).into()
                    }
                    Err(msg) => {
                        warn!("{}", msg);
                        continue;
                    }
                },
                None => ContextStyleAttribute::new(None, attribute.1.clone()).into(),
            };

            if !attributes.iter().any(|csa| csa.logical_eq(&new_entry)) {
                attributes.push(new_entry);
            } else {
                warn!("Style overwritten for {:?}", new_entry);
                // Safe unwrap: checked in if above
                let index = attributes
                    .iter()
                    .position(|csa| csa.logical_eq(&new_entry))
                    .unwrap();
                attributes[index] = new_entry;
            }
        }

        DynamicStyle::copy_from(attributes)
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

impl EntityCommand for SetImageTint {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "image tint",
                LockableStyleAttribute::ImageTint
            );
        }

        // TODO: bevy 0.14: Wire to UiImage.color
        if let Some(mut backgroun_color) = world.get_mut::<BackgroundColor>(entity) {
            if backgroun_color.0 != self.image_tint {
                backgroun_color.0 = self.image_tint;
            }
        } else {
            world
                .entity_mut(entity)
                .insert(BackgroundColor(self.image_tint));
        }
    }
}

impl EntityCommand for SetImageFlip {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "image flip",
                LockableStyleAttribute::ImageFlip
            );
        }

        let Some(mut image) = world.get_mut::<UiImage>(entity) else {
            warn!(
                "Failed to set image flip on entity {:?}: No UiImage component found!",
                entity
            );
            return;
        };

        if image.flip_x != self.image_flip.x {
            image.flip_x = self.image_flip.x;
        }

        if image.flip_y != self.image_flip.y {
            image.flip_y = self.image_flip.y;
        }
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

impl EntityCommand for SetIcon {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "icon", LockableStyleAttribute::Image);
            // TODO: Check lock on text / font once it is available
        }

        // TODO: Rework once text/font is in better shape
        match self.icon {
            IconData::None => {
                world.entity_mut(entity).remove::<Text>();
                world.entity_mut(entity).remove::<UiImage>();
            }
            IconData::Image(path, color) => {
                SetImage {
                    path,
                    check_lock: self.check_lock,
                }
                .apply(entity, world);
                SetImageTint {
                    image_tint: color,
                    check_lock: self.check_lock,
                }
                .apply(entity, world);
            }
            IconData::FontCodepoint(font, codepoint, color, font_size) => {
                SetImageTint {
                    image_tint: Color::NONE,
                    check_lock: self.check_lock,
                }
                .apply(entity, world);

                world.entity_mut(entity).remove::<UiImage>();
                let font = world.resource::<AssetServer>().load(font);

                if let Some(mut text) = world.get_mut::<Text>(entity) {
                    text.sections = vec![TextSection::new(
                        codepoint,
                        TextStyle {
                            font,
                            font_size,
                            color,
                        },
                    )];
                } else {
                    world.entity_mut(entity).insert((
                        Text::from_section(
                            codepoint,
                            TextStyle {
                                font,
                                font_size,
                                color,
                            },
                        )
                        .with_justify(JustifyText::Center)
                        .with_no_wrap(),
                        TextLayoutInfo::default(),
                        TextFlags::default(),
                    ));
                }
            }
        }
    }
}

// TODO: Update these once font / text handling improves
impl EntityCommand for SetFont {
    fn apply(self, entity: Entity, world: &mut World) {
        let font = world.resource::<AssetServer>().load(self.font);

        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set font on entity {:?}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = text
            .sections
            .iter_mut()
            .map(|section| {
                section.style.font = font.clone();
                section.clone()
            })
            .collect();
    }
}

impl EntityCommand for SetFontSize {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set font on entity {:?}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = text
            .sections
            .iter_mut()
            .map(|section| {
                section.style.font_size = self.font_size;
                section.clone()
            })
            .collect();
    }
}

impl EntityCommand for SetSizedFont {
    fn apply(self, entity: Entity, world: &mut World) {
        let font = world.resource::<AssetServer>().load(self.sized_font.font);

        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set sized font on entity {:?}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = text
            .sections
            .iter_mut()
            .map(|section| {
                section.style.font = font.clone();
                section.style.font_size = self.sized_font.size;
                section.clone()
            })
            .collect();
    }
}

impl EntityCommand for SetFontColor {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set font on entity {:?}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = text
            .sections
            .iter_mut()
            .map(|section| {
                section.style.color = self.font_color;
                section.clone()
            })
            .collect();
    }
}

struct SetLockedAttribute {
    attribute: LockableStyleAttribute,
    locked: bool,
}

impl EntityCommand for SetLockedAttribute {
    fn apply(self, entity: Entity, world: &mut World) {
        if let Some(mut locked_attributes) = world.get_mut::<LockedStyleAttributes>(entity) {
            if self.locked {
                if !locked_attributes.contains(self.attribute) {
                    locked_attributes.0.insert(self.attribute);
                }
            } else {
                if locked_attributes.contains(self.attribute) {
                    locked_attributes.0.remove(&self.attribute);
                }
            }
        } else if self.locked {
            let mut locked_attributes = LockedStyleAttributes::default();
            locked_attributes.0.insert(self.attribute);
            world.entity_mut(entity).insert(locked_attributes);
        }
    }
}

pub trait SetLockedAttributeExt<'a> {
    fn lock_attribute(&'a mut self, attribute: LockableStyleAttribute) -> &mut UiStyle<'a>;
}

impl<'a> SetLockedAttributeExt<'a> for UiStyle<'a> {
    fn lock_attribute(&'a mut self, attribute: LockableStyleAttribute) -> &mut UiStyle<'a> {
        self.commands.add(SetLockedAttribute {
            attribute,
            locked: true,
        });
        self
    }
}

pub trait SetLockedAttributeUncheckedExt<'a> {
    fn unlock_attribute(
        &'a mut self,
        attribute: LockableStyleAttribute,
    ) -> &mut UiStyleUnchecked<'a>;
}

impl<'a> SetLockedAttributeUncheckedExt<'a> for UiStyleUnchecked<'a> {
    fn unlock_attribute(
        &'a mut self,
        attribute: LockableStyleAttribute,
    ) -> &mut UiStyleUnchecked<'a> {
        self.commands.add(SetLockedAttribute {
            attribute,
            locked: false,
        });
        self
    }
}

impl EntityCommand for SetScale {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "scale", LockableStyleAttribute::Scale);
        }

        let Some(mut transform) = world.get_mut::<Transform>(entity) else {
            warn!(
                "Failed to set scale on entity {:?}: No Transform component found!",
                entity
            );
            return;
        };

        let new_scale = Vec3::ONE * self.scale;
        if transform.scale != new_scale {
            transform.scale = new_scale;
        }
    }
}
