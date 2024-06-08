use bevy::{ecs::system::EntityCommand, prelude::*, ui::FocusPolicy};

use sickle_macros::StyleCommands;

use crate::{flux_interaction::FluxInteraction, theme::prelude::*};

use super::{
    attribute::{
        ApplyCustomAnimatadStyleAttribute, ApplyCustomInteractiveStyleAttribute,
        ApplyCustomStaticStyleAttribute, CustomAnimatedStyleAttribute,
        CustomInteractiveStyleAttribute, CustomStaticStyleAttribute, InteractiveVals,
    },
    builder::{AnimatedStyleBuilder, InteractiveStyleBuilder, StyleBuilder},
    manual::{ImageSource, SetAbsolutePositionExt, SetFluxInteractionExt, SetImageExt},
    AnimatedVals, LockedStyleAttributes, LogicalEq, TrackedStyleState, UiStyle, UiStyleUnchecked,
};

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
        image: ImageSource,
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
    #[target_enum]
    #[skip_lockable_enum]
    #[animatable]
    TrackedStyleState {
        tracked_style_state: TrackedStyleState,
    },
    #[skip_lockable_enum]
    #[skip_enity_command]
    #[animatable]
    Size {
        size: Val,
    },
}
