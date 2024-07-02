use bevy::{ecs::system::EntityCommand, prelude::*, text::TextLayoutInfo, ui::widget::TextFlags};

use crate::{flux_interaction::FluxInteraction, theme::icons::IconData};

use super::{
    generated::*, LockableStyleAttribute, LockedStyleAttributes, UiStyle, UiStyleUnchecked,
};

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

#[derive(Clone, Debug)]
pub enum ImageSource {
    Path(String),
    Lookup(String, fn(String, Entity, &mut World) -> Handle<Image>),
    Handle(Handle<Image>),
}

impl Default for ImageSource {
    fn default() -> Self {
        Self::Handle(Handle::default())
    }
}

impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        Self::Path(path.to_string())
    }
}

impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        Self::Path(path)
    }
}

pub struct SetImage {
    source: ImageSource,
    check_lock: bool,
}

impl EntityCommand for SetImage {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "image", LockableStyleAttribute::Image);
        }

        let handle = match self.source {
            ImageSource::Path(path) => {
                if path == "" {
                    Handle::default()
                } else {
                    world.resource::<AssetServer>().load(path)
                }
            }
            ImageSource::Lookup(path, callback) => callback(path, entity, world),
            ImageSource::Handle(handle) => handle,
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

pub trait SetImageExt {
    fn image(&mut self, source: ImageSource) -> &mut Self;
}

impl SetImageExt for UiStyle<'_> {
    fn image(&mut self, source: ImageSource) -> &mut Self {
        self.commands.add(SetImage {
            source,
            check_lock: true,
        });
        self
    }
}

pub trait SetImageUncheckedExt {
    fn image(&mut self, source: ImageSource) -> &mut Self;
}

impl SetImageUncheckedExt for UiStyleUnchecked<'_> {
    fn image(&mut self, source: ImageSource) -> &mut Self {
        self.commands.add(SetImage {
            source,
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

pub struct SetFluxInteractionEnabled {
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

pub trait SetFluxInteractionExt {
    fn disable_flux_interaction(&mut self) -> &mut Self;
    fn enable_flux_interaction(&mut self) -> &mut Self;
    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self;
}

impl SetFluxInteractionExt for UiStyle<'_> {
    fn disable_flux_interaction(&mut self) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: false,
            check_lock: true,
        });
        self
    }

    fn enable_flux_interaction(&mut self) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: true,
            check_lock: true,
        });
        self
    }

    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled,
            check_lock: true,
        });
        self
    }
}

pub trait SetFluxInteractionUncheckedExt {
    fn disable_flux_interaction(&mut self) -> &mut Self;
    fn enable_flux_interaction(&mut self) -> &mut Self;
    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self;
}

impl SetFluxInteractionUncheckedExt for UiStyleUnchecked<'_> {
    fn disable_flux_interaction(&mut self) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: false,
            check_lock: false,
        });
        self
    }

    fn enable_flux_interaction(&mut self) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled: true,
            check_lock: false,
        });
        self
    }

    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self {
        self.commands.add(SetFluxInteractionEnabled {
            enabled,
            check_lock: false,
        });
        self
    }
}

pub trait SetNodeShowHideExt {
    fn show(&mut self) -> &mut Self;
    fn hide(&mut self) -> &mut Self;
    fn render(&mut self, render: bool) -> &mut Self;
}

impl SetNodeShowHideExt for UiStyle<'_> {
    fn show(&mut self) -> &mut Self {
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

    fn hide(&mut self) -> &mut Self {
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

    fn render(&mut self, render: bool) -> &mut Self {
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

pub trait SetNodeShowHideUncheckedExt {
    fn show(&mut self) -> &mut Self;
    fn hide(&mut self) -> &mut Self;
    fn render(&mut self, render: bool) -> &mut Self;
}

impl SetNodeShowHideUncheckedExt for UiStyleUnchecked<'_> {
    fn show(&mut self) -> &mut Self {
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

    fn hide(&mut self) -> &mut Self {
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

    fn render(&mut self, render: bool) -> &mut Self {
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

pub struct SetAbsolutePosition {
    absolute_position: Vec2,
    check_lock: bool,
}

impl EntityCommand for SetAbsolutePosition {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
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

            let size = parent_node.unrounded_size();
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

        style.top = Val::Px(self.absolute_position.y - offset.y);
        style.left = Val::Px(self.absolute_position.x - offset.x);
    }
}

pub trait SetAbsolutePositionExt {
    fn absolute_position(&mut self, position: Vec2) -> &mut Self;
}

impl SetAbsolutePositionExt for UiStyle<'_> {
    fn absolute_position(&mut self, position: Vec2) -> &mut Self {
        self.commands.add(SetAbsolutePosition {
            absolute_position: position,
            check_lock: true,
        });
        self
    }
}

pub trait SetAbsolutePositionUncheckedExt {
    fn absolute_position(&mut self, position: Vec2) -> &mut Self;
}

impl SetAbsolutePositionUncheckedExt for UiStyleUnchecked<'_> {
    fn absolute_position(&mut self, position: Vec2) -> &mut Self {
        self.commands.add(SetAbsolutePosition {
            absolute_position: position,
            check_lock: false,
        });
        self
    }
}

impl EntityCommand for SetIcon {
    fn apply(self, entity: Entity, world: &mut World) {
        // TODO: Rework once text/font is in better shape
        match self.icon {
            IconData::None => {
                if self.check_lock {
                    check_lock!(world, entity, "icon", LockableStyleAttribute::Image);
                    // TODO: Check lock on text / font once it is available
                }
                SetImageTint {
                    image_tint: Color::NONE,
                    check_lock: self.check_lock,
                }
                .apply(entity, world);
                world.entity_mut(entity).remove::<Text>();
                world.entity_mut(entity).remove::<UiImage>();
            }
            IconData::Image(path, color) => {
                SetImage {
                    source: ImageSource::Path(path),
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
                // TODO: Check lock on text / font once it is available

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

pub trait SetLockedAttributeExt {
    fn lock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self;
}

impl SetLockedAttributeExt for UiStyle<'_> {
    fn lock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self {
        self.commands.add(SetLockedAttribute {
            attribute,
            locked: true,
        });
        self
    }
}

pub trait SetLockedAttributeUncheckedExt {
    fn unlock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self;
}

impl SetLockedAttributeUncheckedExt for UiStyleUnchecked<'_> {
    fn unlock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self {
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

impl EntityCommand for SetSize {
    fn apply(self, entity: Entity, world: &mut World) {
        if self.check_lock {
            check_lock!(world, entity, "size: width", LockableStyleAttribute::Width);
            check_lock!(
                world,
                entity,
                "size: height",
                LockableStyleAttribute::Height
            );
        }

        let Some(mut style) = world.get_mut::<Style>(entity) else {
            warn!(
                "Failed to set size on entity {:?}: No Style component found!",
                entity
            );
            return;
        };

        if style.width != self.size {
            style.width = self.size;
        }

        if style.height != self.size {
            style.height = self.size;
        }
    }
}
