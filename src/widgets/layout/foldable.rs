use bevy::{prelude::*, ui::FocusPolicy};

use sickle_ui_scaffold::prelude::*;

use crate::widgets::{menus::menu_item::MenuItemUpdate, WidgetLibraryUpdate};

use super::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
    panel::UiPanelExt,
};

pub struct FoldablePlugin;

impl Plugin for FoldablePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            FoldableUpdate
                .after(MenuItemUpdate)
                .before(WidgetLibraryUpdate),
        )
        .add_plugins(ComponentThemePlugin::<Foldable>::default())
        .add_systems(
            Update,
            (handle_foldable_button_press, update_foldable_container)
                .chain()
                .in_set(FoldableUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct FoldableUpdate;

fn handle_foldable_button_press(
    mut q_foldables: Query<(&mut Foldable, &FluxInteraction), Changed<FluxInteraction>>,
) {
    for (mut foldable, interaction) in &mut q_foldables {
        if interaction.is_released() {
            foldable.open = !foldable.open;

            // Only process a maximum of one foldable in a frame
            break;
        }
    }
}

fn update_foldable_container(
    q_foldables: Query<(Entity, &Foldable), Changed<Foldable>>,
    mut commands: Commands,
) {
    for (entity, foldable) in &q_foldables {
        if foldable.empty {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Empty)
                .remove_pseudo_state(PseudoState::Folded);

            continue;
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Empty);
        }

        if foldable.open {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Folded);
        } else {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Folded);
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Foldable {
    pub open: bool,
    pub empty: bool,
    icon: Entity,
    label: Entity,
    container: Entity,
}

impl Default for Foldable {
    fn default() -> Self {
        Self {
            open: Default::default(),
            empty: Default::default(),
            icon: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            container: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for Foldable {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Foldable::BUTTON_ICON => Ok(self.icon),
            Foldable::BUTTON_LABEL => Ok(self.label),
            Foldable::CONTAINER => Ok(self.container),
            _ => Err(format!(
                "{} doesn't exists for Foldable. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![
            Foldable::BUTTON_ICON,
            Foldable::BUTTON_LABEL,
            Foldable::CONTAINER,
        ]
    }
}

impl DefaultTheme for Foldable {
    fn default_theme() -> Option<Theme<Foldable>> {
        Foldable::theme().into()
    }
}

impl Foldable {
    pub const BUTTON_ICON: &'static str = "ButtonIcon";
    pub const BUTTON_LABEL: &'static str = "ButtonLabel";
    pub const CONTAINER: &'static str = "Container";

    pub fn theme() -> Theme<Foldable> {
        let base_theme = PseudoTheme::deferred(None, Foldable::primary_style);
        let folded_theme = PseudoTheme::deferred(vec![PseudoState::Folded], Foldable::folded_style);
        let empty_theme = PseudoTheme::deferred(vec![PseudoState::Empty], Foldable::empty_style);

        Theme::new(vec![base_theme, folded_theme, empty_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .switch_target(Foldable::BUTTON_ICON)
            .size(Val::Px(theme_spacing.icons.small))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .icon(
                theme_data
                    .icons
                    .expand_more
                    .with(colors.on(On::Surface), theme_spacing.icons.small),
            )
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Foldable::BUTTON_LABEL)
            .margin(UiRect::right(Val::Px(theme_spacing.gaps.medium)))
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(On::SurfaceVariant),
                hover: colors.on(On::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Foldable::CONTAINER)
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    fn folded_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder.switch_target(Foldable::BUTTON_ICON).icon(
            theme_data
                .icons
                .chevron_right
                .with(colors.on(On::Surface), theme_spacing.icons.small),
        );

        style_builder
            .switch_target(Foldable::CONTAINER)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    }

    fn empty_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder.switch_target(Foldable::BUTTON_ICON).icon(
            theme_data
                .icons
                .arrow_right
                .with(colors.on(On::Surface), theme_spacing.icons.small),
        );

        style_builder
            .switch_target(Foldable::CONTAINER)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    }

    pub fn container(&self) -> Entity {
        self.container
    }

    fn button(name: String) -> impl Bundle {
        (
            Name::new(format!("Foldable [{}] - Button", name)),
            ButtonBundle {
                background_color: Color::NONE.into(),
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            TrackedInteraction::default(),
        )
    }

    fn button_icon() -> impl Bundle {
        (Name::new("Fold Icon"), ImageBundle::default())
    }
}

pub trait UiFoldableExt<'w> {
    fn foldable<'a>(
        &'a mut self,
        name: impl Into<String>,
        open: bool,
        empty: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity>;
}

impl<'w> UiFoldableExt<'w> for UiBuilder<'w, '_, Entity> {
    fn foldable<'a>(
        &'a mut self,
        name: impl Into<String>,
        open: bool,
        empty: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 'a, Entity> {
        let name = name.into();

        let mut foldable = Foldable {
            open,
            empty,
            ..default()
        };

        let button = self
            .container(Foldable::button(name.clone()), |button| {
                foldable.icon = button.spawn(Foldable::button_icon()).id();
                foldable.label = button
                    .label(LabelConfig {
                        label: name.clone(),
                        ..default()
                    })
                    .id();
            })
            .id();

        foldable.container = self.panel(name, spawn_children).id();
        if !open {
            self.commands().style(foldable.container).hide();
            self.commands()
                .entity(button)
                .add_pseudo_state(PseudoState::Folded);
        }

        self.commands().entity(button).insert(foldable);
        self.commands().ui_builder(button)
    }
}
