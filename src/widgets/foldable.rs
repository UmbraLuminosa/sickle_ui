use bevy::prelude::*;

use crate::{
    ui_builder::*,
    ui_style::{SetFlexGrowExt, SetNodeShowHideExt, UiStyleExt},
};

use super::{
    prelude::{MenuItem, MenuItemConfig, MenuItemUpdate, UiContainerExt, UiMenuItemExt, UiRowExt},
    WidgetLibraryUpdate,
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
    mut q_menu_items: Query<(&MenuItem, &mut Foldable), Changed<MenuItem>>,
) {
    for (menu_item, mut foldable) in &mut q_menu_items {
        if menu_item.interacted() {
            if foldable.open {
                foldable.open = false;
            } else {
                foldable.open = true;
            }

            // Only process a maximum of one foldable in a frame
            break;
        }
    }
}

fn update_foldable_container(
    mut q_menu_items: Query<(&mut MenuItemConfig, &Foldable), Changed<Foldable>>,
    mut commands: Commands,
) {
    for (mut config, foldable) in &mut q_menu_items {
        if foldable.open {
            config.leading_icon = Some("embedded://sickle_ui/icons/chevron_down.png".into());
            commands.style(foldable.container).show();
        } else {
            config.leading_icon = Some("embedded://sickle_ui/icons/chevron_right.png".into());
            commands.style(foldable.container).hide();
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Foldable {
    pub open: bool,
    container: Entity,
}

impl Default for Foldable {
    fn default() -> Self {
        Self {
            open: Default::default(),
            container: Entity::PLACEHOLDER,
        }
    }
}

impl Foldable {
    pub fn container(&self) -> Entity {
        self.container
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                justify_items: JustifyItems::Start,
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiFoldableExt<'w, 's> {
    fn foldable<'a>(
        &'a mut self,
        name: impl Into<String>,
        open: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiFoldableExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn foldable<'a>(
        &'a mut self,
        name: impl Into<String>,
        open: bool,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let mut button = Entity::PLACEHOLDER;
        let container;
        let name = name.into();

        self.row(|row| {
            button = row
                .menu_item(MenuItemConfig {
                    name: name.clone(),
                    leading_icon: Some("embedded://sickle_ui/icons/chevron_right.png".into()),
                    ..default()
                })
                .style()
                .flex_grow(1.)
                .id();
        })
        .insert(Name::new(format!("Foldable [{}] - Button Row", name)));

        container = self
            .container(
                (
                    Name::new(format!("Foldable [{}] - Container", name)),
                    Foldable::frame(),
                ),
                spawn_children,
            )
            .id();
        if !open {
            self.commands().style(container).hide();
        }

        self.commands()
            .entity(button)
            .insert(Foldable { container, open });

        self.commands().ui_builder(button)
    }
}
