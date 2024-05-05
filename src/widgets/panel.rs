use bevy::prelude::*;
use sickle_macros::UiContext;

use crate::{
    theme::{
        dynamic_style::DynamicStyle, theme_colors::Surface, theme_data::ThemeData,
        ComponentThemePlugin, PseudoTheme, Theme, UiContext,
    },
    ui_builder::UiBuilder,
    ui_style::StyleBuilder,
};

use super::prelude::UiContainerExt;

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<Panel>::default());
    }
}

#[derive(Component, Clone, Debug, Reflect, UiContext)]
#[reflect(Component)]
pub struct Panel {
    own_id: Entity,
    pub title: String,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            own_id: Entity::PLACEHOLDER,
            title: "".into(),
        }
    }
}

impl Default for Theme<Panel> {
    fn default() -> Self {
        Panel::theme()
    }
}

impl Panel {
    pub fn own_id(&self) -> Entity {
        self.own_id
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn theme() -> Theme<Panel> {
        let base_theme = PseudoTheme::deferred(None, Panel::container);
        Theme::<Panel>::new(vec![base_theme])
    }

    fn container(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder
            .width(Val::Percent(100.))
            .height(Val::Percent(100.))
            .flex_direction(FlexDirection::Column)
            .background_color(theme_data.colors().surface(Surface::Surface));
    }

    fn frame() -> impl Bundle {
        let style: DynamicStyle = ThemeData::with_default(Panel::container).into();
        (NodeBundle::default(), style)
    }
}

pub trait UiPanelExt<'w, 's> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiPanelExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn panel<'a>(
        &'a mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'w, 's, 'a, Entity> {
        let name = format!("Panel [{}]", title.clone());
        let mut container = self.container((Name::new(name), Panel::frame()), spawn_children);
        let own_id = container.id();

        container.insert(Panel {
            own_id,
            title,
            ..default()
        });

        container
    }
}
