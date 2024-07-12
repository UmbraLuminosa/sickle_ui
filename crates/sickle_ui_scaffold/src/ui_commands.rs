use std::marker::PhantomData;

use bevy::{
    core::Name,
    ecs::{
        component::ComponentInfo,
        entity::Entity,
        query::With,
        system::{Commands, EntityCommand, EntityCommands},
        world::{Command, World},
    },
    hierarchy::{Children, Parent},
    log::{info, warn},
    state::state::{FreelyMutableState, NextState, States},
    text::{Text, TextSection, TextStyle},
    ui::Interaction,
    window::{CursorIcon, PrimaryWindow, Window},
};

use crate::{
    flux_interaction::{
        FluxInteraction, FluxInteractionStopwatchLock, StopwatchLock, TrackedInteraction,
    },
    prelude::UiUtils,
    theme::prelude::*,
    ui_style::builder::StyleBuilder,
};

struct SetTextSections {
    sections: Vec<TextSection>,
}

impl EntityCommand for SetTextSections {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set text sections on entity {}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = self.sections;
    }
}

pub trait SetTextSectionsExt {
    fn set_text_sections(&mut self, sections: Vec<TextSection>) -> &mut Self;
}

impl SetTextSectionsExt for EntityCommands<'_> {
    fn set_text_sections(&mut self, sections: Vec<TextSection>) -> &mut Self {
        self.add(SetTextSections { sections });
        self
    }
}

struct SetText {
    text: String,
    style: TextStyle,
}

impl EntityCommand for SetText {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set text on entity {}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = vec![TextSection::new(self.text, self.style)];
    }
}

pub trait SetTextExt {
    fn set_text(&mut self, text: impl Into<String>, style: Option<TextStyle>) -> &mut Self;
}

impl SetTextExt for EntityCommands<'_> {
    fn set_text(&mut self, text: impl Into<String>, style: Option<TextStyle>) -> &mut Self {
        self.add(SetText {
            text: text.into(),
            style: style.unwrap_or_default(),
        });

        self
    }
}

struct UpdateText {
    text: String,
}

impl EntityCommand for UpdateText {
    fn apply(self, entity: Entity, world: &mut World) {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set text on entity {}: No Text component found!",
                entity
            );
            return;
        };

        let first_section = match text.sections.get(0) {
            Some(section) => TextSection::new(self.text, section.style.clone()),
            None => TextSection::new(self.text, TextStyle::default()),
        };

        text.sections = vec![first_section];
    }
}

pub trait UpdateTextExt {
    fn update_text(&mut self, text: impl Into<String>) -> &mut Self;
}

impl UpdateTextExt for EntityCommands<'_> {
    fn update_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.add(UpdateText { text: text.into() });

        self
    }
}

// TODO: Move to style and apply to Node's window
struct SetCursor {
    cursor: CursorIcon,
}

impl Command for SetCursor {
    fn apply(self, world: &mut World) {
        let mut q_window = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = q_window.get_single_mut(world) else {
            return;
        };

        if window.cursor.icon != self.cursor {
            window.cursor.icon = self.cursor;
        }
    }
}

pub trait SetCursorExt<'w, 's, 'a> {
    fn set_cursor(&mut self, cursor: CursorIcon);
}

impl<'w, 's, 'a> SetCursorExt<'w, 's, 'a> for Commands<'w, 's> {
    fn set_cursor(&mut self, cursor: CursorIcon) {
        self.add(SetCursor { cursor });
    }
}

struct LogHierarchy {
    level: usize,
    is_last: bool,
    trace_levels: Vec<usize>,
    component_filter: Option<fn(ComponentInfo) -> bool>,
}

impl EntityCommand for LogHierarchy {
    fn apply(self, id: Entity, world: &mut World) {
        let mut children_ids: Vec<Entity> = Vec::new();
        if let Some(children) = world.get::<Children>(id) {
            children_ids = children.iter().map(|child| *child).collect();
        }

        let filter = self.component_filter;
        let debug_infos: Vec<_> = world
            .inspect_entity(id)
            .into_iter()
            .filter(|component_info| {
                if let Some(filter) = filter {
                    filter((*component_info).clone())
                } else {
                    true
                }
            })
            .map(UiUtils::simplify_component_name)
            .collect();

        let prefix = if self.is_last { "╚" } else { "╠" };
        let mut padding_parts: Vec<&str> = Vec::with_capacity(self.level);
        for i in 0..self.level {
            let should_trace = i > 0 && self.trace_levels.contains(&(i - 1));

            padding_parts.push(match should_trace {
                true => "  ║ ",
                false => "    ",
            });
        }

        let padding = padding_parts.join("");
        let name = match world.get::<Name>(id) {
            Some(name) => format!("[{}] {}", id, name),
            None => format!("Entity {}", id),
        };
        let entity_text = format!("{}  {}══ {} ", padding, prefix, name);
        let has_children = children_ids.len() > 0;

        info!("{}", entity_text);
        for i in 0..debug_infos.len() {
            let is_last = i == (debug_infos.len() - 1);
            let component_pipe = if is_last { "└" } else { "├" };
            let child_pipe = if self.is_last {
                if has_children {
                    "      ║      "
                } else {
                    "             "
                }
            } else {
                if has_children {
                    "  ║   ║      "
                } else {
                    "  ║          "
                }
            };
            info!(
                "{}{}{}── {}",
                padding, child_pipe, component_pipe, debug_infos[i]
            );
        }

        if children_ids.len() > 0 {
            let next_level = self.level + 1;

            for i in 0..children_ids.len() {
                let child = children_ids[i];
                let is_last = i == (children_ids.len() - 1);
                let mut trace_levels = self.trace_levels.clone();
                if !is_last {
                    trace_levels.push(self.level);
                }

                LogHierarchy {
                    level: next_level,
                    is_last,
                    trace_levels,
                    component_filter: self.component_filter,
                }
                .apply(child, world);
            }
        }
    }
}

pub trait LogHierarchyExt {
    fn log_hierarchy(&mut self, component_filter: Option<fn(ComponentInfo) -> bool>) -> &mut Self;
}

impl LogHierarchyExt for EntityCommands<'_> {
    /// Logs the hierarchy of the entity along with the component of each entity in the tree.
    /// Components listed can be optionally filtered by supplying a `component_filter`
    ///
    /// ## Example
    /// ``` rust
    /// commands.entity(parent_id).log_hierarchy(Some(|info| {
    ///     info.name().contains("Node")
    /// }));
    /// ```
    /// ## Output Example
    /// ```
    /// ╚══ Entity 254v2:
    ///     ║      └── Node
    ///     ╠══ Entity 252v2:
    ///     ║   ║      └── Node
    ///     ║   ╚══ Entity 158v2:
    ///     ║       ║      └── Node
    ///     ║       ╠══ Entity 159v2:
    ///     ║       ║   ║      └── Node
    ///     ║       ║   ╚══ Entity 286v1:
    ///     ║       ║              └── Node
    ///     ║       ╚══ Entity 287v1:
    ///     ║                  └── Node
    ///     ╚══ Entity 292v1:
    ///                └── Node
    /// ```
    fn log_hierarchy(&mut self, component_filter: Option<fn(ComponentInfo) -> bool>) -> &mut Self {
        self.add(LogHierarchy {
            level: 0,
            is_last: true,
            trace_levels: vec![],
            component_filter,
        });
        self
    }
}

// Adopted from @brandonreinhart
pub trait EntityCommandsNamedExt {
    fn named(&mut self, name: impl Into<String>) -> &mut Self;
}

impl EntityCommandsNamedExt for EntityCommands<'_> {
    fn named(&mut self, name: impl Into<String>) -> &mut Self {
        self.insert(Name::new(name.into()))
    }
}

pub trait RefreshThemeExt {
    fn refresh_theme<C>(&mut self) -> &mut Self
    where
        C: DefaultTheme;
}

impl RefreshThemeExt for EntityCommands<'_> {
    fn refresh_theme<C>(&mut self) -> &mut Self
    where
        C: DefaultTheme,
    {
        self.add(RefreshEntityTheme::<C> {
            context: PhantomData,
        });
        self
    }
}

struct RefreshEntityTheme<C>
where
    C: DefaultTheme,
{
    context: PhantomData<C>,
}

impl<C> EntityCommand for RefreshEntityTheme<C>
where
    C: DefaultTheme,
{
    fn apply(self, entity: Entity, world: &mut World) {
        let context = world.get::<C>(entity).unwrap();
        let theme_data = world.resource::<ThemeData>().clone();
        let pseudo_states = world.get::<PseudoStates>(entity);
        let empty_pseudo_state = Vec::new();

        let pseudo_states = match pseudo_states {
            Some(pseudo_states) => pseudo_states.get(),
            None => &empty_pseudo_state,
        };

        // Default -> General (App-wide) -> Specialized (Screen) theming is a reasonable guess.
        // Round to 4, which is the first growth step.
        // TODO: Cache most common theme count in theme data.
        let mut themes: Vec<(&Theme<C>, Option<Entity>)> = Vec::with_capacity(4);
        // Add own theme
        if let Some(own_theme) = world.get::<Theme<C>>(entity) {
            themes.push((own_theme, Some(entity)));
        }

        // Add all ancestor themes
        let mut current_ancestor = entity;
        while let Some(parent) = world.get::<Parent>(current_ancestor) {
            current_ancestor = parent.get();
            if let Some(ancestor_theme) = world.get::<Theme<C>>(current_ancestor) {
                themes.push((ancestor_theme, Some(current_ancestor)));
            }
        }

        let default_theme = C::default_theme();
        if let Some(ref default_theme) = default_theme {
            themes.push((default_theme, None));
        }

        if themes.len() == 0 {
            warn!(
                "Theme missing for component {} on entity: {}",
                std::any::type_name::<C>(),
                entity
            );
            return;
        }

        // The list contains themes in reverse order of application
        themes.reverse();

        // Assuming we have a base style and two-three pseudo state style is a reasonable guess.
        // TODO: Cache most common pseudo theme count in theme data.
        let mut pseudo_themes: Vec<(&PseudoTheme<C>, Option<Entity>)> =
            Vec::with_capacity(themes.len() * 4);

        for (theme, source_entity) in &themes {
            if let Some(base_theme) = theme.pseudo_themes().iter().find(|pt| pt.is_base_theme()) {
                pseudo_themes.push((base_theme, *source_entity));
            }
        }

        if pseudo_states.len() > 0 {
            for i in 0..pseudo_states.len() {
                for (theme, source_entity) in &themes {
                    theme
                        .pseudo_themes()
                        .iter()
                        .filter(|pt| pt.count_match(pseudo_states) == i + 1)
                        .for_each(|pt| pseudo_themes.push((pt, *source_entity)));
                }
            }
        }

        // Merge base attributes on top of the default and down the chain, overwriting per-attribute at each level
        let styles: Vec<(Option<Entity>, DynamicStyle)> = pseudo_themes
            .iter()
            .map(
                |(pseudo_theme, source_entity)| match pseudo_theme.builder() {
                    DynamicStyleBuilder::Static(style) => vec![(None, style.clone())],
                    DynamicStyleBuilder::StyleBuilder(builder) => {
                        let mut style_builder = StyleBuilder::new();
                        builder(&mut style_builder, &theme_data);

                        style_builder.convert_with(context)
                    }
                    DynamicStyleBuilder::ContextStyleBuilder(builder) => {
                        let mut style_builder = StyleBuilder::new();
                        builder(&mut style_builder, &context, &theme_data);

                        style_builder.convert_with(context)
                    }
                    DynamicStyleBuilder::WorldStyleBuilder(builder) => {
                        let mut style_builder = StyleBuilder::new();
                        builder(&mut style_builder, entity, &context, world);

                        style_builder.convert_with(context)
                    }
                    DynamicStyleBuilder::InfoWorldStyleBuilder(builder) => {
                        let mut style_builder = StyleBuilder::new();
                        builder(
                            &mut style_builder,
                            *source_entity,
                            pseudo_theme.state(),
                            entity,
                            &context,
                            world,
                        );

                        style_builder.convert_with(context)
                    }
                },
            )
            .filter(|e_to_dys| e_to_dys.len() > 0)
            .fold(
                Vec::with_capacity(context.contexts().len() + 1),
                |mut acc, context_styles| {
                    for context_style in context_styles {
                        let index = acc.iter().position(|entry| entry.0 == context_style.0);
                        match index {
                            Some(index) => {
                                let (_, prev_entry) = acc[index].clone();
                                acc[index].1 = prev_entry.merge(context_style.1);
                            }
                            None => acc.push(context_style),
                        }
                    }

                    acc
                },
            );

        let mut cleanup_main_style = true;
        let mut unstyled_entities: Vec<Entity> = context
            .cleared_contexts()
            .iter()
            .map(|ctx_name| {
                // Unsafe unwrap: ctx_name comes from the context itslef, we should panic if it doesn't resolve!
                context.get(&ctx_name).unwrap()
            })
            .collect();

        for (placement, mut style) in styles {
            let placement_entity = match placement {
                Some(placement_entity) => placement_entity,
                None => {
                    cleanup_main_style = false;
                    entity
                }
            };

            unstyled_entities.retain(|e| *e != placement_entity);

            if let Some(current_style) = world.get::<DynamicStyle>(placement_entity) {
                style.copy_controllers(current_style);
            }

            if style.is_interactive() || style.is_animated() {
                if world.get::<Interaction>(placement_entity).is_none() {
                    world
                        .entity_mut(placement_entity)
                        .insert(Interaction::default());
                }

                if world.get_mut::<FluxInteraction>(placement_entity).is_none() {
                    world
                        .entity_mut(placement_entity)
                        .insert(TrackedInteraction::default());
                }
            }

            world.entity_mut(placement_entity).insert(style);
        }

        for unstyled_context in unstyled_entities {
            world.entity_mut(unstyled_context).remove::<DynamicStyle>();
        }

        if cleanup_main_style {
            world.entity_mut(entity).remove::<DynamicStyle>();
        }
    }
}

pub trait ManageFluxInteractionStopwatchLockExt {
    fn lock_stopwatch(&mut self, owner: &'static str, duration: StopwatchLock) -> &mut Self;

    fn try_release_stopwatch_lock(&mut self, lock_of: &'static str) -> &mut Self;
}

impl ManageFluxInteractionStopwatchLockExt for EntityCommands<'_> {
    fn lock_stopwatch(&mut self, owner: &'static str, duration: StopwatchLock) -> &mut Self {
        self.add(move |entity, world: &mut World| {
            if let Some(mut lock) = world.get_mut::<FluxInteractionStopwatchLock>(entity) {
                lock.lock(owner, duration);
            } else {
                let mut lock = FluxInteractionStopwatchLock::new();
                lock.lock(owner, duration);
                world.entity_mut(entity).insert(lock);
            }
        });
        self
    }

    fn try_release_stopwatch_lock(&mut self, lock_of: &'static str) -> &mut Self {
        self.add(move |entity, world: &mut World| {
            if let Some(mut lock) = world.get_mut::<FluxInteractionStopwatchLock>(entity) {
                lock.release(lock_of);
            }
        });
        self
    }
}

pub trait ManagePseudoStateExt {
    fn add_pseudo_state(&mut self, state: PseudoState) -> &mut Self;
    fn remove_pseudo_state(&mut self, state: PseudoState) -> &mut Self;
}

impl ManagePseudoStateExt for EntityCommands<'_> {
    fn add_pseudo_state(&mut self, state: PseudoState) -> &mut Self {
        self.add(move |entity, world: &mut World| {
            let pseudo_states = world.get_mut::<PseudoStates>(entity);

            if let Some(mut pseudo_states) = pseudo_states {
                // NOTE: we must check here, as calling the add fn will trigger change detection
                if !pseudo_states.has(&state) {
                    pseudo_states.add(state);
                }
            } else {
                let mut pseudo_states = PseudoStates::new();
                pseudo_states.add(state);

                world.entity_mut(entity).insert(pseudo_states);
            }
        });
        self
    }

    fn remove_pseudo_state(&mut self, state: PseudoState) -> &mut Self {
        self.add(move |entity, world: &mut World| {
            let Some(mut pseudo_states) = world.get_mut::<PseudoStates>(entity) else {
                return;
            };

            // NOTE: we must check here, as calling the remove fn will trigger change detection
            if pseudo_states.has(&state) {
                pseudo_states.remove(state);
            }
        });

        self
    }
}

pub trait UpdateStatesExt<'w, 's, 'a> {
    fn next_state<C: States + FreelyMutableState>(&mut self, state: C);
}

impl<'w, 's, 'a> UpdateStatesExt<'w, 's, 'a> for Commands<'w, 's> {
    fn next_state<C: States + FreelyMutableState>(&mut self, state: C) {
        self.add(|world: &mut World| {
            if let Some(mut old_state) = world.get_resource_mut::<NextState<C>>() {
                old_state.set(state);
            } else {
                warn!(
                    "Failed to set state: {}, state not initialized!",
                    std::any::type_name::<C>()
                );
            }
        });
    }
}
