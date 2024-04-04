use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy::{ecs::system::EntityCommand, ui::FocusPolicy};
use sickle_macros::StyleCommands;

use crate::{
    ui_style::{UiStyle, UiStyleUnchecked},
    FluxInteraction,
};

#[derive(Component, Debug, Default)]
pub struct LockedStyleAttributes(HashSet<StylableAttribute>);

impl LockedStyleAttributes {
    pub fn contains(&self, attr: StylableAttribute) -> bool {
        self.0.contains(&attr)
    }
}

// TODO: Add support for continous animations, i.e. loop, ping-pong
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AnimationProgress {
    #[default]
    Start,
    Inbetween(f32),
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InteractionAnimationState {
    phase: crate::FluxInteraction,
    iteration: u8,
    animation: AnimationProgress,
}

pub struct StaticValueBundle<T: Clone> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
}

impl<T: Clone> StaticValueBundle<T> {
    fn to_value(&self, flux_interaction: crate::FluxInteraction) -> T {
        self.base.clone()
    }
}

pub struct AnimatedValueBundle<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> {
    base: T,
    hover: Option<T>,
    press: Option<T>,
    cancel: Option<T>,
    focus: Option<T>,
}

impl<T: sickle_math::lerp::Lerp + Default + Clone + Copy + PartialEq> AnimatedValueBundle<T> {
    fn to_value(
        &self,
        transition_base: InteractionAnimationState,
        animation_progress: InteractionAnimationState,
    ) -> T {
        // TODO: LERP
        self.base
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
    // #[skip_enity_command]
    // ZIndex {
    //     z_index: ZIndex,
    // },
    // #[skip_ui_style_ext]
    // Image {
    //     image: String,
    // },
    // #[skip_enity_command]
    // ImageScaleMode {
    //     image_scale_mode: ImageScaleMode,
    // },
}
