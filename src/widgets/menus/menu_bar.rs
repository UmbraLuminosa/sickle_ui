use crate::ui_builder::*;
use bevy::prelude::*;

use sickle_macros::UiContext;
use sickle_ui_scaffold::{
    theme::{
        theme_colors::Container, theme_data::ThemeData, ComponentThemePlugin, DefaultTheme,
        PseudoTheme, Theme, UiContext,
    },
    ui_style::{LockableStyleAttribute, LockedStyleAttributes, StyleBuilder},
};

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MenuBar>::default());
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MenuBar;

impl DefaultTheme for MenuBar {
    fn default_theme() -> Option<Theme<MenuBar>> {
        MenuBar::theme().into()
    }
}

impl MenuBar {
    pub fn theme() -> Theme<MenuBar> {
        let base_theme = PseudoTheme::deferred(None, MenuBar::primary_style);
        Theme::<MenuBar>::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Percent(100.))
            .height(Val::Px(theme_spacing.areas.medium))
            .background_color(colors.container(Container::SurfaceMid))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Menu Bar"),
            NodeBundle {
                style: Style {
                    overflow: Overflow::visible(),
                    ..default()
                },
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::Overflow),
        )
    }
}

pub trait UiMenuBarExt<'w, 's> {
    fn menu_bar<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, MenuBar)>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiMenuBarExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn menu_bar<'a>(
        &'a mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, MenuBar)>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let id = self.spawn((MenuBar::frame(), MenuBar)).id();

        let mut builder = self.commands().ui_builder((id, MenuBar));
        spawn_children(&mut builder);

        self.commands().ui_builder(id)
    }
}

pub trait UiMenuBarSubExt<'w, 's> {
    fn id(&self) -> Entity;
}

impl<'w, 's> UiMenuBarSubExt<'w, 's> for UiBuilder<'w, 's, '_, (Entity, MenuBar)> {
    fn id(&self) -> Entity {
        self.context().0
    }
}
