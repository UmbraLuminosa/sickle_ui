use bevy::{asset::embedded_asset, prelude::*};

pub(crate) struct BuiltInIconsPlugin;

impl Plugin for BuiltInIconsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/assets/", "checkmark.png");
        embedded_asset!(app, "src/assets/", "chevron_down.png");
        embedded_asset!(app, "src/assets/", "chevron_left.png");
        embedded_asset!(app, "src/assets/", "chevron_right.png");
        embedded_asset!(app, "src/assets/", "chevron_up.png");
        embedded_asset!(app, "src/assets/", "close.png");
        embedded_asset!(app, "src/assets/", "details_menu.png");
        embedded_asset!(app, "src/assets/", "exit.png");
        embedded_asset!(app, "src/assets/", "exit_white.png");
        embedded_asset!(app, "src/assets/", "menu_icon_default.png");
        embedded_asset!(app, "src/assets/", "menu_icon_default_48.png");
        embedded_asset!(app, "src/assets/", "menu_icon_selected.png");
        embedded_asset!(app, "src/assets/", "menu_icon_selected_48.png");
        embedded_asset!(app, "src/assets/", "popout_white.png");
        embedded_asset!(app, "src/assets/", "redo.png");
        embedded_asset!(app, "src/assets/", "redo_white.png");
        embedded_asset!(app, "src/assets/", "submenu.png");
        embedded_asset!(app, "src/assets/", "checkmark.png");
        embedded_asset!(app, "src/assets/", "submenu_white.png");
        embedded_asset!(app, "src/assets/", "tiles_menu.png");
        embedded_asset!(app, "src/assets/", "undo.png");
    }
}
