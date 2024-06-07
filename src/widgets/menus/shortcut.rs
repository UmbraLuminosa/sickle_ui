use bevy::prelude::*;

use crate::input_extension::SymmetricKeysExt;

pub struct ShortcutPlugin;

impl Plugin for ShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreUpdate, ShortcutPreUpdate)
            .add_systems(
                PreUpdate,
                (reset_pressed_shortcuts, update_shortcut_on_key_press)
                    .chain()
                    .in_set(ShortcutPreUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ShortcutPreUpdate;

fn reset_pressed_shortcuts(mut q_shortcuts: Query<&mut Shortcut>) {
    for mut shortcut in &mut q_shortcuts {
        if shortcut.pressed {
            shortcut.bypass_change_detection().pressed = false;
        }
    }
}

fn update_shortcut_on_key_press(
    mut q_shortcuts: Query<&mut Shortcut>,
    r_keys: Res<ButtonInput<KeyCode>>,
) {
    if !r_keys.is_changed() {
        return;
    }

    for mut shortcut in &mut q_shortcuts {
        if shortcut.code.len() == 0 {
            continue;
        }

        let main_key = shortcut.code.last().unwrap().clone();
        if r_keys.just_pressed(main_key) {
            if shortcut.code.len() > 1 {
                if shortcut
                    .code
                    .iter()
                    .take(shortcut.code.len() - 1)
                    .map(|c| c.clone())
                    .all(|keycode| r_keys.symmetry_pressed(keycode))
                {
                    shortcut.pressed = true;
                }
            } else {
                shortcut.pressed = true;
            }
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Shortcut {
    code: Vec<KeyCode>,
    pressed: bool,
}

impl Shortcut {
    pub fn new(code: Vec<KeyCode>) -> Self {
        Self {
            code,
            pressed: false,
        }
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }
}
