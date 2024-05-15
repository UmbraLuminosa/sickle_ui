use bevy::prelude::*;

use super::*;

pub struct InteractiveStyleBuilder<'a> {
    pub style_builder: &'a mut StyleBuilder,
}

pub struct AnimatedStyleBuilder<'a> {
    pub style_builder: &'a mut StyleBuilder,
}

impl<'a> AnimatedStyleBuilder<'a> {
    pub fn add_and_extract_animation(
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
        value.attributes.iter().for_each(|attr| {
            if let Some(context) = &attr.0 {
                warn!(
                    "StyleBuilder with context-bound attributes converted without context! \
                    [{}] attributes discarded! \
                    This can be the result of using `PseudoTheme::build()` and calling \
                    `style_builder.switch_context(CONTEXT)` in the callback, which is not supported.",
                    context
                );
            }
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
