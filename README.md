# Sickle UI

[![Crates.io](https://img.shields.io/crates/v/sickle_ui.svg)](https://crates.io/crates/sickle_ui)
[![Downloads](https://img.shields.io/crates/d/sickle_ui.svg)](https://crates.io/crates/sickle_ui)

A widget library built on top of `bevy`'s internal `bevy_ui`.

![Screenshot of the simple_editor example](/assets/images/Screenshot_Simple_Editor.png)

## Example

If you clone the repository, you can simply build and run the main example:

```
cargo build
cargo run --example simple_editor
```

> [!WARNING]
> `sickle_ui` is still in development. The framework is stable to the extend listed below.

Main missing features:
- Centralized focus management
- Text / Text area input widgets

What it can already do:
- Resizable layout
  - Rows / columns
  - Scroll views
  - Docking zones
  - Tab containers
  - Floating panels
  - Sized zones
  - Foldables
- Input
  - Slider
  - Dropdown
  - Checkbox
  - Radio groups
- Menu
  - Menu item (with leading/trailing icons and support for keyboard shortcuts)
  - Toggle menu item
  - Submenu
  - Context menu (component-based)
- Static
  - Icon
  - Label
- Utility
  - Command-based styling
  - Temporal tracking of interactions
  - Animated interactions
  - Context based extensions
  - Drag / drop interactions
  - Scroll interactions
- Theming
  - Material 3 based color scheme (dark/light, 3 contrast levels per theme)
  - Centralized sizing control
  - Centralized font control
  - Automatic theme updates
  - Theme overrides


## Getting started

First you need to add `sickle_ui` as a dependency to your project:

```toml
[dependencies]
sickle_ui = "0.2.1"
# sickle_ui = { rev = "a548517", git = "https://github.com/UmbraLuminosa/sickle_ui" }
```

> [!NOTE]
> Use the commented out line and change `rev = "..."` to a version of your chosing if you want
> to depend on the repository directly. Major versions are marked with a git tag.

Once you have the new dependency, `cargo build` to download it. Now you are ready to use it,
so add it to your app as a plugin:

```rust
use bevy::prelude::*;
use sickle_ui::{prelude::*, SickleUiPlugin};

fn main() {
  App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SickleUiPlugin)
        // ... your actual app plugins and systems can go here of course
        .run();
}
```

The main `SickleUiPlugin` takes care of adding all the convenient features `sickle_ui` offers, and
the `sickle_ui::prelude::*` brings into scope all available extensions. Have a look at the `simple_editor`
example (that is displayed in the screenshot above) for how different parts work together.


## Foreword

> [!IMPORTANT]
> Sickle UI is primarily using `Commands` and `EntityCommands` to spawn, style, and configure widgets.
> Systems using these widgets need to consider that the changes will not be reflected in the ECS `world`
> until the next `apply_deferred` is executed. This is mostly automatic starting from `bevy 0.13`. Internally
> `sickle_ui` uses systems in well defined sets and order to make sure all widgets play nicely with each other.


## Basic use case

In the most simple use cases you just want to use existing widgets to build up your UI. Sickle UI adds
extensions to both `Commands` and `EntityCommands`, so in a regular system context you can quickly
create a layout by calling a chain of functions. Comparing vanilla and Sickle UI:


### Vanilla `bevy`

In Bevy, you can use `commands.spawn(bundle)` and `commands.entity(entity).with_children(builder)` to
spawn your entities. Typically, you would pass in a `NodeBundle`, `ButtonBundle`, or perhaps an
`ImageBundle` just to name a few. Then, you can use the `.with_children(builder)` extension to spawn
sub-entities. This will quickly become verbose and convulated with Rust's borrowing rules. It will
be difficult to create entities with two way references between parent and children, elements further
down the tree, or siblings.

```rust
fn setup(mut commands: Commands) {
  commands.spawn(NodeBundle {
      style: Style {
          height: Val::Percent(100.),
          flex_direction: FlexDirection::Column,
          ..default()
      },
      background_color: Color::NONE.into(),
      ..default()
  }).with_children(|parent|{
    parent.spawn(NodeBundle::default()).with_children(|parent|{
      // ...
    });
  });
}
```


### Sickle UI

The library takes care of this by abstracting widget creation behind builder extensions such as:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.row(|row|{
      // ... etc.
    });
  });
}
```

While this may seem as a simple shorthand, the key difference is that `column` and `row` in the
callbacks are contextual builders themselves and they give you access to `commands` and, where
available, `entity_commands`. You can easily jump to another entity to insert components,
style, or spawn new sub-entitites without tripping Rust's borrow checker.


### Did I mention style?

Yes, you can also style entities spawned by the command chains, as simple as:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    // ...
  })
  .style()
  .width(Val::Percent(100.));
}
```

> [!NOTE]
> The return value of a builder function can be different from the internal builder. A
> good example would be `scroll_view`, where the external return value is the builder of the frame
> (outermost) entity, while the internal builder is for its content view (that will be clipped to
> the frame!).

> [!NOTE]
> Styling interactions is not possible this way. These are only static styles. See 
> [StyleBuilder](#style-builder) on how to apply interactive styling.

This means that in _some_ cases, this also works as expected:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column
      .style()
      .width(Val::Percent(100.));
  });
}
```

The difference in the above is merely a styling choice of the developer (pun intended).

> [!IMPORTANT]
> Styling is applied as a regular command in the chain, so rendering of the component
> will change the next time UI layout is calulated by `bevy` in its `PostUpdate` systems. The `style`
> commands are mapped to the `Style` component fields and _some other_ component fields that affect
> the overall display of the `Node`, such as `BackgroundColor`, `BorderColor`, etc.

> [!WARNING]
> Theming may override styles applied this way. Read [Theming](#theming) further down on how theming works.


### Noteworthy contexts

As mentioned, all builder function have a context.
- The root one is `UiRoot`. The entity spawned in the `UiRoot` context does not have a `Parent` entity,
hence it will be a root `Node`.
- The most common regular context is `Entity`, which can be acquired by calling `commands.ui_builder(entity)`.
Where `entity` is an entity - ID - acquired by some other means, such as spawning or querying.

> [!TIP]
> Other contexts are specific for use cases, such as the tab container's or that of the menu system. You'll
> find these eventually as you use these widgets, but they are generally transparent. Use the editor's
> auto-complete feature to see what extensions are available in each!

> [!CAUTION]
> `UiRoot` must not be confused with `UiContextRoot`. The former is a marker to indicate that we spawn without
> a `Parent` while the latter is a component that indicates the _logical_ root of a sub-tree of widgets.
> It is used by widgets such as `ContextMenu` and `TabContainer` to find mounting points for dynamically
> spawned widgets. `ContextMenu` places the menu container at `UiContextRoot` and `TabContainer` creates
> the `FloatingPanel` at this location in the tree when a tab is popped out.


### Alright, now I want to find my `column`

Fear not your column, or any other widget you create can be used like any other entity you have around.
To just add a component:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|_|{}).insert(MyMarkerComponent);
}
```

You could capture its ID as well:

```rust
fn setup(mut commands: Commands) {
  let my_column = commands.ui_builder(UiRoot).column(|_|{}).id();

  // ... OR

  let my_column = commands.ui_builder(UiRoot).column(|_|{}).insert(MyMarkerComponent).id();
}
```

> [!TIP]
> The same applies here as with styling. Callbacks may point to the same entity as the frame, so
> `insert` may be called in the callback as well:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.insert(MyMarkerComponent); 
  });
}
```


### OK, but I didn't find a widget I need

If you just need a simple bundle somewhere in the tree, you can either use `spawn` or a container widget, like `container` to create or chain your one-off node. So, converting the `bevy` example we started with:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.container(NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }, |my_container|{
      // ... etc. my_container is an `Entity` context UiBuilder
    });
  });
}
```

If you do not even need to spawn children for this widget:

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.spawn(NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    });
  });
}
```

> [!TIP]
> Since we are using `Commands` and `EntityCommands` and just spawn regular `bevy_ui` `Node`s
> you can also mix this syntax with the vanilla Bevy spawns:

```rust
fn setup(mut commands: Commands) {
  let mut inner_id = Entity::PLACEHOLDER;
  
  commands.spawn(NodeBundle {
      style: Style {
          height: Val::Percent(100.),
          flex_direction: FlexDirection::Column,
          ..default()
      },
      background_color: Color::NONE.into(),
      ..default()
  }).with_children(|parent|{
    inner_id = parent.spawn(NodeBundle::default()).with_children(|parent|{
      // ...
    }).id();
  });

  commands.ui_builder(inner_id).column(|column|{
    // Add a column into the inner entity and continue.
  });
}
```

> [!TIP]
> And vica-versa!

```rust
fn setup(mut commands: Commands) {
  commands.ui_builder(UiRoot).column(|column|{
    column.row(|row|{
      let mut row_commands = row.entity_commands();
      row_commands.with_children(|parent| {
        // ... etc.
      });
    });
  });
}
```


### OK, but my widget isn't _simple_

Then you shall move on to the next section, [Extending Sickle UI](#extending-sickle-ui)!


## Extending Sickle UI

Sickle UI can be extended on multiple levels. Starting from the most simple one:

- Structural extensions
- Functional extensions
- Themed widgets
- Contextually themed widgets

These are however, NOT distinct extensions. Rather these are levels of customization you can apply
to the widgets you create. If you don't need dynamic theming, you don't need to implement all that.

> [!TIP]
> `sickle_ui` includes a snippet for each of the scenarios outlined above to get you started.
> These are VSCode snippets, available in the `.vscode` folder. You can either copy the
> `sickle_ui.code-snippets` to your workspace's `.vscode` folder, or copy the file contents to your
> Rust snippets (`File` -> `Preferences` -> `Configure User Snippets` -> `[select the rust language from the list]`)


### Structural extensions

These are widgets that don't need systems and just create a pre-defined sub-tree that you can easily inject
in the contexts you define them in. In this case you just need to create the relevant extension and describe
your plugin structure using the technique described under [OK, but I didn't find a widget I need](#ok-but-i-didnt-find-a-widget-i-need)

An example of this would be:

```rust
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MyWidget;

impl MyWidget {
    fn frame() -> impl Bundle {
        (Name::new("My Widget"), NodeBundle::default())
    }
}

pub trait UiMyWidgetExt {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container((MyWidget::frame(), MyWidget), spawn_children)
    }
}
```

> [!TIP]
> The above has been generated with the snippet `Sickle UI Widget` available if you start typing `sickle` in
> an `.rs` file in VSCode (if you have added the snippets). You can customize the suggestion trigger in the
> snippet files, but it is recommended to avoid using `widget` as a trigger (it collides with often used `width`).

> [!TIP]
> The snippets support 3 tab points: The widget component name, the convenience `Name` component string,
> and the actual extension function name.

You can then use your widget after bringing it into scope:

```rust
use my_widget::UiMyWidgetExt;

fn setup(mut commands: Commands) {
  // TODO: get your root entity where your widget will be added.
  // This could come from a query for example.
  let root_entity: Entity;
  commands.ui_builder(root_entity).my_widget(|my_widget|{
    // ... do more here!
  });
}
```

You may have noticed that the snippet extends the `Entity` context of `UiBuilder`. Your widget will be
available in these contexts, provided you add the `use my_widget::UiMyWidgetExt;` to bring it in scope.

> [!TIP]
> VSCode with the regular Rust extensions is smart enouth to _suggest_ the import if you type out the
> extension name and press `Ctrl + .` (or the Mac equivalent `Command + .`).

You may have _also_ noticed that the snippet uses `self` to spawn the `container`. `self` will simply be
a `UiBuilder` of the `Entity` context, so any _other_ extensions that you brought into scope with `use`
will be available. This also means that `style` commands are also available, so long as you have imported them.


### Functional extension

Functional extension simply means that your widget _does_ something beyond creating a pre-defined structure.
You can use the snippet `Sickle UI plugin widget` to generate code similar to the one outlined in
[Structural extensions](#structural-extensions), with the addition of a plugin:

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, _app: &mut App) {
        // TODO
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MyWidget;

impl MyWidget {
    fn frame() -> impl Bundle {
        (Name::new("My Widget"), NodeBundle::default())
    }
}

pub trait UiMyWidgetExt {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container((MyWidget::frame(), MyWidget), spawn_children)
    }
}
```

> [!TIP]
> The snippets also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

All that is left is for you to implement the heart of the widget and the systems that act on it. Don't
forget to add the generated plugin to your app!


### Themed widgets

Now, this is where the fun begins.

Themed widgets refer to widgets that have a style defined for them in a central place. However, themed widgets 
also allow overrides to their style, based on their position in the widget tree or their [pseudo states](#pseudo-states).

> [!IMPORTANT]
> Themed widgets only apply style to their outermost `Node`, but not to their sub-nodes. Those are the
> [Contextually themed widgets](#contextually-themed-widgets).

Similarly to the previous cases, there is a snippet to generate the shell of a themed widget:
The `Sickle UI themed plugin widget`.

> [!TIP]
> The snippets also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MyWidget;

impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}

impl MyWidget {
    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

    fn frame() -> impl Bundle {
        (Name::new("My Widget"), NodeBundle::default())
    }
}

pub trait UiMyWidgetExt {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        self.container((MyWidget::frame(), MyWidget), spawn_children)
    }
}
```

While we have seen most of the above from the previous snippets, there are a couple additions.


#### The ComponentThemePlugin

First, an additional plugin has been injected to our app in the widget's plugin definition:

```rust
impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        // This here is very important!
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}
```

The `ComponentThemePlugin` handles theme calculation and reloading for the component is added for.
In this case we added it for `MyWidget`, which is the example component.

> [!IMPORTANT]
> `MyWidget` now ***must*** derive `UiContext`. This derive provides default implementation for the context
> we will look at later in [Contextually themed widgets](#contextually-themed-widgets).

Next, we have the implementation of `DefaultTheme`:


#### The `DefaultTheme`

```rust
impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}
```

This is the theme that will be applied (unless it returns `None`) to any widget in the widget tree that has no
overrides on any of its ancestors. We will look at how this works exactly in the [Theming](#theming) section.

For now, the key point is that it is generally desirable to implement the default theme of the widget as part
of this implementation so an explicit injection is not needed or a sane fallback is provided.

The last part is the actual definition of the theme as part of the widget's `impl` block:

```rust
impl MyWidget {
    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

    // ...
}
```

The two function above define the theme itself and the styling that is applied as part of the
[PseudoTheme](#pseudo-theme) of `None`. This is simply the style that is applied when the widget has no special
[PseudoState](#pseudo-states) attached to it. It is the base theme and the fallback style that is always applied
to any new entities that are added to the widget tree. It is also the basis of any overrides.

In the simplest use case, defining the style is just a matter of calling style function on the
provided `style_builder`. The methods available here are the same as the ones provided by the `UiStyle`
extensions outlined in [Did I mention style?](#did-i-mention-style) with a few additions.

> [!TIP]
> See [Style builder](#style-builder) further below for information on what it provides.

With this, we have a convenient place to implement all our styling needs.

> [!IMPORTANT]
> Styles defined in a theme are applied in `PostUpdate` as part of the `DynamicStylePostUpdate` system set.
> This means that any style the node was created with (as overrides in the spawn bundle) or those that were
> applied via `.style()` commands will potentially be overwritten here.


### Contextually themed widgets

Contextually themed widgets take [Themed widgets](#themed-widgets) a step further by allowing the styling to be
applied to sub-widgets defined as part of the main widget. The snippet `Sickle UI contexted themed plugin widget`
generates the following shell:

```rust
pub struct MyWidgetPlugin;

impl Plugin for MyWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MyWidget>::default());
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MyWidget {
    label: Entity,
}

impl Default for MyWidget {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for MyWidget {
    fn default_theme() -> Option<Theme<MyWidget>> {
        MyWidget::theme().into()
    }
}

impl UiContext for MyWidget {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MyWidget::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for MyWidget. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![MyWidget::LABEL]
    }
}

impl MyWidget {
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<MyWidget> {
        let base_theme = PseudoTheme::deferred(None, MyWidget::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));

        style_builder
            .switch_target(MyWidget::LABEL)
            .sized_font(font);
    }

    fn frame() -> impl Bundle {
        (Name::new("My Widget"), NodeBundle::default())
    }
}

pub trait UiMyWidgetExt {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let label = self
            .label(LabelConfig {
                label: "MyWidget".into(),
                ..default()
            })
            .id();

        self.container((MyWidget::frame(), MyWidget { label }), spawn_children)
    }
}
```

> [!TIP]
> The snippet also supports tab points, so you can quickly name the widget and plugin in a consistent manner.

Now, our widget component is no longer just a tag. It now has a reference to a label sub-widget:

```rust
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MyWidget {
    label: Entity,
}

impl Default for MyWidget {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}

// ...

impl UiMyWidgetExt for UiBuilder<'_, Entity> {
    fn my_widget(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let label = self
            .label(LabelConfig {
                label: "MyWidget".into(),
                ..default()
            })
            .id();

        self.container((MyWidget::frame(), MyWidget { label }), spawn_children)
    }
}
```

We need to implement `Default` for it manually, since `Entity` has no default. Using `Entity::PLACEHOLDER` is
alright as long as we make sure we always assign an actual entity to it (otherwise it will panic!).

#### The UiContext

But this isnt't the only addition. Now our snippet defined an implementation for `UiContext` we previously
got from a simple `derive`:

```rust
impl UiContext for MyWidget {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            MyWidget::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exists for MyWidget. Possible contexts: {:?}",
                target,
                self.contexts()
            )),
        }
    }

    fn contexts(&self) -> Vec<&'static str> {
        vec![MyWidget::LABEL]
    }
}
```

This tells the theming system that `MyWidget` has a single additional context (besides the main entity).
The additional context can be accessed by the `MyWidget::LABEL` constant, which was added to the `impl` block:

```rust
impl MyWidget {
    pub const LABEL: &'static str = "Label";

    // ...
}
```

Further down we can also see a change: The `primary_style` now applies styling to the label!

```rust
impl MyWidget {
    // ...

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));

        style_builder
            .switch_target(MyWidget::LABEL)
            .sized_font(font);
    }

    // ...
}
```

In the above code, there is a call on `style_builder` to `switch_target` to our label and set its font size.
Refer to [Style builder](#style-builder) for how this works in detail.

> [!CAUTION]
> Once a target is set, all subsequent calls to `style_builder` will be applied to the target.
> You can `reset_target` on the builder to swap to the main widget again, but it is more readable
> to have each target in a single chain / group.


### That's it?

In a nutshell, yes. If you use the snippets, you can quickly set up a complex widget tree and define each
sub-widget's style by chaining the calls to the `style_builder`. Of course, there are other ways to interact
with the theming process, such as accesing the world or the current widget component, but the heart of it is
the same: A theme, made up of pseudo themes that build the styling of the widget and its sub-widgets.


## Theming

Theming is the process of applying styling on an entity (`Node`) based on its position in the widget tree,
function, and current state. A [Theme](#theme) is a collection of [PseudoTheme](#pseudo-theme)s, which define
the style for an entity when it has the relevant `PseudoState`s in its [PseudoStates](#pseudo-states)
collection component.

Styling is done per-attribute, meaning each stylable attribute has its own entry in the final
[DynamicStyle](#dynamic-style) component attached to the entity. Each [Theme](#theme) and their
[PseudoTheme](#pseudo-theme)s are evaluated in a strict order to calculate the final style for each attribute.

> [!IMPORTANT]
> [DynamicStyle](#dynamic-style) components can be generated and attached to entities manually as well.
> This is useful if the developer would like to have the _power_ of the interactive / animated styling,
> but do not wish to pay the cost of the theming lookup in general. It is also useful for one-off widgets.

> [!CAUTION]
> `DynamicStyle` components are _removed_ by the theming system for any entity marked in the `UiContext`'s
> `cleared_contexts`. By default, this is the whole list of its `contexts`.


### Evaluation order

When a themed component is added to the hierarchy, the system will look for all [Theme](#theme) components
in its chain of ancestors (including itself) until it reaches a root entity. [DefaultTheme](#the-defaulttheme)
implementations are checked last. Once the list of applicable [Theme](#theme)s are found, they are evaluated
in reverse order. This means that the [DefaultTheme](#the-defaulttheme) is the first that will be evaluated,
then any override starting from the root entity, down to the themed entity itself.

Once we have the list of [Theme](#theme)s, each theme is expanded to collect the applicable
[PseudoTheme](#pseudo-theme)s in their order of `specificity`. A [PseudoTheme](#pseudo-theme) is considered
if, and only if, all of the `PseudoState`s it was defined for is on the entity. However, if it only defines
a subset of `PseudoState`s it will still be considered, but before the ones that fully cover the states.

> [!NOTE]
> `specificity` is the number of `PseudoState`s that the [PseudoTheme](#pseudo-theme) was defined for.
> The only exception is the case when a [PseudoTheme](#pseudo-theme) was defined for `None`, which is
> considered the base pseudo theme of the entity.


#### Exmaple:

If an entity has the `PseudoState`s `[Checked, Disabled, FirstChild]` then [PseudoTheme](#pseudo-theme)s
defined for `None`, `[Checked]`, `[Disabled]`, `[FirstChild]`, `[Checked, Disabled]`, `[Checked, FirstChild]`,
and `[Checked, Disabled, FirstChild]` will be considered, in this order.

If the entity only has the `[Checked]` state, then [PseudoTheme](#pseudo-theme)s defined for `None`, and
`[Checked]` will be applied, but none of the others because they are either defined for a disjoint set or
they are not a complete subset of the entity's state.

> [!IMPORTANT]
> The [PseudoTheme](#pseudo-theme) defined for `None` or the empty set of `[]` are considered the base
> pseudo themes. This means that they will always be applied before any of the more specific
> [PseudoTheme](#pseudo-theme)s.

> [!CAUTION]
> When the `specificity` of a [PseudoTheme](#pseudo-theme) is the same as an other pseudo theme, they will
> be applied in the order they were added to the [Theme](#theme)!


### What triggers theming?

If the [ComponentThemePlugin::<C>](#the-componentthemeplugin) is in place, the following changes trigger
themes to be processed for the managed component `C`:

- Entity added with `C`: The theme for each new entity will be evaluated and applied
- [Theme data](#theme-data) resource changed: All entities with `C` will be processed
- Any `Theme<C>` added, changed, or removed: All entities with `C` will be processed
- Any entity with component `C` will be re-processed if their [PseudoStates](#pseudo-states) changes
(or if it has been removed).

> [!TIP]
> In case the `ComponentThemePlugin` was not used, theme processing can be manually triggered by calling
> `commands.entity(entity).refresh_theme::<C>();`.

> [!TIP]
> `ComponentThemePlugin` can be set to be a "custom" theme. This simply means that the processing will
> take place in a system set called `CustomThemeUpdate`. This set is scheduled between `ThemeUpdate` and
> `DynamicStyleUpdate` and lets developers alter or use the outcome of regular theming steps.


### Can I use CSS?

No.


#### But technically, if I write my own parser?

Still no. Themes are related to components, and there is no theme merging across components. This is because
`sickle_ui` does not support defining relation between component themes to achieve this (for multiple reasons).

HOWEVER! If we are talking about the case where developers no longer use the `C` in `CSS` it is possible.

Modern web development usually follows some sort of style simplification to avoid running into issues with
ambigous specificities or the performance cost of deeply nested styles (not to mention minimization). One
widespread method is to use the BEM (Block, Element, Modifier) notation to compose class names. Combined with
a pre-processor like SASS and some discipline, most single page apps have a single-level nested style sheet.

Parsing such a style sheet, generating a `bevy` component for each of the classes, then transforming the style
to themes should be entirely possible. Because of how theme overrides work some nesting can also be achieved so
long as two themes don't style the same _entity_. To make this work well, the brave developer would need to
implement:

- A setup that removes nesting from raw CSS (BEM + SASS is a good starting point)
- Something* to parse the above mentioned "flat" CSS to generate components and themes
- Systems that automatically inject theme overrides to achieve nesting
- And finally some systems to automatically apply `PseudoState`s matching that of CSS. See
[PseudoStates](#pseudo-states) for what is already implemented and how to use it.

> [!NOTE]
> To support hot-reloading the parser would need to work with a single, pre-defined component that has information
> on what CSS `class` it corresponds to on any given entity. Themes then can use this information to recover
> the style sheet of such an entity. The scaffolding to allow this approach already exist in `sickle_ui`.


#### Will you..

No.


### Theme

`Theme<C + DefaultTheme>` is a `bevy` component used to hold [PseudoTheme](#pseudo-theme)s. Inserting
a `Theme::<C>` component in the widget tree will override styling for `C` components below (or on) it.


### Pseudo theme

`PseudoTheme<C>` is a carrier struct to map a list of `PseudoState`s to _builders_ for styling. While this
struct can be created directly with a `DynamicStyleBuilder` variant, it is recommended to use one of the
exposed function:

- [PseudoTheme::build](crates/sickle_ui_scaffold/src/theme.rs#L116)
- [PseudoTheme::deferred](crates/sickle_ui_scaffold/src/theme.rs#L129)
- [PseudoTheme::deferred_context](crates/sickle_ui_scaffold/src/theme.rs#L139)
- [PseudoTheme::deferred_world](crates/sickle_ui_scaffold/src/theme.rs#L149)
- [PseudoTheme::deferred_info_world](crates/sickle_ui_scaffold/src/theme.rs#L159)

#### `build`

`build` requires a simple callback that accepts a `StyleBuilder` instance to setup the entity style.
This style builder is immediately evaluated to generate the `DynamicStyle` that will be cloned to
entities. Switching context on the style builder emits a warning. This is because the target context
cannot be known at compile time.


#### `deferred` variants

Deferred builders are stored as callbacks and evaluated when the theming system is applying styling.
Depending on the variant you use, the callback will receive a different set of parameters:

- `deferred` will receive the style builder and the [theme data](#theme-data) resource
- `deferred_context` will additionally receive `&C`, which is a reference to the styled component instance.
- `deferred_world` will receive the entity (ID), `&C`, and a readonly reference to the `World`.
- `deferred_info_world` will additionally receive the ID of the theme that is being applied and the set of
`PseudoState`s. These are both optional as the theming could be done from the `DefaultTheme` for the base
pseudo theme (defined for `None`). This callback is useful if the whole context is needed to map a callback
to an external stylesheet implementation.

> [!IMPORTANT]
> Callbacks may be evaluated even if the final style they generate will be discarded entirely. This is because
> The overrides are calculated per-attribute and not per pseudo theme!


### Pseudo states

`PseudoStates` is a `bevy` component with the sole purpose of holding `PseudoState` variants. This component
is monitored by the [ComponentThemePlugin](#the-componentthemeplugin), see 
[What triggers theming?](#what-triggers-theming) for how it ties into it.

`EntityCommands` extensions are provided with the trait 
[ManagePseudoStateExt](crates/sickle_ui_scaffold/src/ui_commands.rs#L551) to manage the list as follows:

- `add_pseudo_state`: used to add a `PseudoState`
- `remove_pseudo_state`: used to remove a `PseudoState`

There are a couple of systems that automatically apply certain `PseudoState`s to entities, but these are all
opt-in:

- Entities tagged with `FlexDirectionToPseudoState` will be processed to set either 
`PseudoState::LayoutRow` or `PseudoState::LayoutColumn` based on their `Style`'s `flex_direction`.
The update is done in `PostUpdate` before `ThemeUpdate`, so themes will automatically process changes in layout.
- Entities tagged with `VisibilityToPseudoState` will be processed to set or remove `PseudoState::Visible`
from their list of `PseudoStates`. This update considers actual visibility based on the entity's
`Visibility` and `InheritedVisibility`, updated only on changes to either of these. The update is done in
`PostUpdate`, after `VisibilitySystems::VisibilityPropagate`, but before `ThemeUpdate`.
- An entity's position among its siblings with component `C` can be tracked with the
`HierarchyToPseudoState::<C>` plugin. This plugin will set `PseudoState::FirstChild`, `PseudoState::LastChild`,
`PseudoState::NthChild(i)`, `PseudoState::SingleChild`, `PseudoState::EvenChild`, and
`PseudoState::OddChild` as appropriate. This is also done in `PostUpdate` before `ThemeUpdate`.

Most build-in widgets will also set `PseudoState`s based on user interaction, such as a `Dropdown` will set
`PseudoState::Open` when the list of options should be visible, etc.. These are documented on the `UiBuilder` extensions themselves.


#### Oh no! I don't have a pseudo state I can abuse!

Don't you worry, there is a `PseudoState::Custom(String)` specifically for such use cases.


### Style builder

The style builder is the recommended way of generating [DynamicStyle](#dynamic-style) components as it lets
developers chain together style attributes in a consistent fashion. On top of what `UiStyle` extensions allow,
the style builder also adds `interactive` and `animated` properties. These properties tie into interactions
from the user, such as hover, press, etc. to change styled attributes.

The distinction between `interactive` and `animated` is that `interactive` styles apply immediately when an
interaction occurs. `animated` attributes perform some eased interpolation between start and end values to
apply the final style.

> [!NOTE]
> `animated` properties break chaining of attributes. This is because animation properties are controlled in
> the chain instead.

In case you want to use the `StyleBuilder` without the [PseudoTheme](#pseudo-theme) fluff, you can simply
create an instance:

```rust
let mut style_builder = StyleBuilder::new();
style_builder
    .width(Val::Percent(100.))
    .height(Val::Percent(100.));

let dynamic_style: DynamicStyle = style_builder.into();
// Insert the dynamic stlye to your entity as any other `bevy` component.
```

> [!WARNING]
> Switching context on the style builder emits a warning when directly converted to a `DynamicStyle`
> This is because the target context cannot be known at compile time.

> [!NOTE]
> You can use `style_builder.convert_with(&context)` and pass in the relevant component instance reference.
> This will proceed to build the style just like [Theming](#theming) would and will return with a
> `Vec<(Option<Entity>, DynamicStyle)>`. Each `DynamicStyle` in the list is accompanied by an optional
> [placement](#switching-placements) entity. When it is `None`, the `DynamicStyle` should be placed on the
> entity the `&context` component was placed on.


#### Example for interactive styling

```rust
fn interactive_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
    style_builder
        .interactive()
        .width(InteractiveVals {
            idle: Val::Px(100.),
            hover: Val::Px(120.).into(),
            ..default()
        })
        .height(InteractiveVals {
            idle: Val::Px(100.),
            press: Val::Px(120.).into(),
            ..default()
        });
}
```


#### Example for animated styling

```rust
fn animated_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
    let theme_spacing = theme_data.spacing;

    style_builder
        .animated()
        .margin(AnimatedVals {
            idle: UiRect::all(Val::Px(theme_spacing.gaps.medium)),
            hover: UiRect::all(Val::Px(0.)).into(),
            ..default()
        })
        .copy_from(theme_data.interaction_animation);

    style_builder
        .animated()
        .scale(AnimatedVals {
            idle: 1.,
            enter_from: Some(0.),
            ..default()
        })
        .copy_from(theme_data.enter_animation);
}
```

> [!CAUTION]
> `interactive` styles can switch between any variant of enum attributes, such as `Val`, as this is 
> an immediate effect.
> HOWEVER, `animated` attributes need compatible variants for `Lerp` calculations. It is not possible
> to interpolate between `Val::Px(50.)` and `Val::Percent(100.)` without expensive calculations, so
> `sickle_ui` simply doesn't do it out of the box. It doesn't mean the developer cannot do it, but they
> need to use `deferred` callbacks that give access to the `world` and use provided utilities available
> in [UiUtils](#uiutils) to convert one variant to the other during creation of the style (so that the
> variants match). They can also convert it according to their chosen implementation, we don't judge.


#### Switching targets

![checkbox retargeting](/assets/gifs/sickle_ui_checkbox_interaction.gif)

As described in [Contextually themed widgets](#contextually-themed-widgets), widget components can
manually implement [The UiContext](#the-uicontext) to provide additional styling targets / placements.

Switching target on the style builder configures subsequent calls to target an entity other than the main one.
The string labels used to identify targets are mapped to entities during the theme building process. The
targets can only be entity properties of the widget component. To revert back to styling the main widget, call
`reset_target` on the style builder. Internally, the target is an `Option` and `None` always means "target the
entity the `DynamicStyle` component is on".

> [!CAUTION]
> Manually building `DynamicStyle` components allow setting target entities directly, but this is not recommended.

> [!IMPORTANT]
> Switching targets allows proxying interactions from the main component! Both `animated` and `interactive`
> styles consider the interaction of their [placement](#switching-placements), i.e. the entity that holds the
> `DynamicStyle` component. However, if the target of the styling is not the main entity, the interaction will
> show up on the target instead.

An example would be the checkbox, that applies styling to both the box and the label when the whole container is
interacted.


#### Switching placements

Switching placement, as opposed to switching the `target` of styling, changes where the `DynamicStyle`
component will be injected. This means that interactions will be detected on the placement and thus can
narrow down interactive area to sub-entities. Calling `reset_placement` reverts the builder to add
attributes to the `DynamicStyle` of the main entity. Internally `placement` is an `Option`, and `None`
means that new attributes are collected for the main entity being styled.

> [!CAUTION]
> The previously set `target` remains even after the switch. It is thus recommended to use
> [switch_context](#switching-context) on the style builder instead to explicitly set the
> placement and target together. `None` target will continue to target the entity the `DynamicStyle`
> is placed on, hence it is not possible to style the main entity from a sub-widget.

> [!TIP]
> As a (intended) side-effect of the above, it is possible to use interactions on one sub-widget, while
> targeting a different sub-widgets with the feedback styling.


#### Switching context

Switching context combines setting `placement` and `target` together, and is the recommended way to interact
with the builder to change `placement`s. This is because the `target` may remain from a previous call and
cause unwanted styling. Call `reset_context` on the style builder to remove both `placement` and `target` setting.


#### I am missing an attribute I want to style!

Don't worry, `sickle_ui` got you covered!

All attribute types (static, interactive, animated) support a `custom` variant, that can be provided as
a callback. `static` attributes receive the `Entity` and `&mut World` as arguments. `interactive` callbacks
will receive the `Entity`, `FluxInteraction`, and `&mut World`, while `animated` callbacks will receive
`Entity`, `AnimationState`, `&mut World`.

You are free to mess up the whole wide world in these callbacks, no guarantees it won't blow up in your face :D


### Dynamic style

`DynamicStyle` is a `bevy` component used to store the list of attributes that should be applied to the
entity during its `PostUpdate` execution. The system set `DynamicStylePostUpdate` is available for
scheduling purposes and it will always run after theming, but before `UiSystem::Layout`.

Systems acting on this component apply the static styling immediately when the `DynamicStyle` changes, and
will keep track and execute `interactive` and `animated` attribute application.

> [!NOTE]
> `DynamicStyle` will clean itself up as best it can. This means that a stylesheet with only staticly
> styled attributes will be removed after the style has been applied. `interactive` and `animated`
> styles will force the component to be kept.

> [!WARNING]
> `animated` styles can have a controller that is set to delete the attribute after the enter animation
> has been executed. This is useful if the enter animation targets a property later controlled by the
> component's own systems (i.e. FloatingPanel size).


### Theme data

Theme data is an opinionated struct containing values useful for general purpose UI styling. Currently,
it has colors loosely matching a Material3 theme, a set of gap sizes, area sizes, input sizes, font
configuration, etc.

Any `sickle_ui` widget that has variable values will depend on the default theme data.

> [!CAUTION]
> Updating the `ThemeData` resource will trigger all themes to be re-evaluated!

> [!NOTE]
> It is gently recommended that any widgets you create for editor purposes (or even games!) to use the
> provided theme data resource. It is a great way to keep the UI widgets consistent across the application.

> [!TIP]
> Custom values are supported as deemed useful in the `ThemeData` struct to store app-specific exceptions.


## Utilities

There are a number of utilities that form the foundation of `sickle_ui` widgets and can be reused for
new widgets just as well.


### FluxInteraction & FluxInteractionStopwatch

`FluxInteraction` is the basis of all interactivity in `sickle_ui`. It is a wrapper around the built-in
`Interaction` but instead of tracking current _state_ it tracks _transitions_ from one state to another.

`FluxInteractionStopwatch` keeps track of the time elapsed since the last change to the `FluxInteraction`
of an entity. This stopwatch by default is available for 1 second after every change for performance reasons.
The resource `FluxInteractionConfig` can be used to control the time it is available, though it is not
recommended to change this. Instead, `FluxInteractionStopwatchLock`s can be used to extend or precise the
time the stopwatch needs to be available.

The recommended way of adding `FluxInteraction` to an entity is by using the provided `TrackedInteraction`
bundle.

> [!NOTE]
> `FluxInteraction` relies on `Interaction`, but will not add it to entities. It has to be added as needed.
> This is because the various built-in `bevy` bundles may or may not add `Interaction` themselves.


### ScrollInteraction

Scroll interactions can be intercepted by the `Scrollable` component attached to an entity. Systems relying
on it can use `Changed<Scrollable>` as a filter to optimize updates.

> [!NOTE]
> Currently only mouse scrolls are supported and the `shift` key is used to alter the scroll axis.


### DragInteraction

The `Draggable` component is built around `FluxInteraction` and `RelativeCursorPosition` to translate 
interactions to drag intents. The component itself doesn't do anything besides tracking the drag intents.
Developers wishing to utilize it can rely on change detection and use the provided values to update aspects
of their widgets, i.e. scroll bars can be dragged to scroll content, resize handles can be dragged to change
the size of their parent, etc.


### DropInteraction

`Draggable`, `Droppable`, and `DropZone` together forms the drop interactions. When a `Droppable` is being
dragged over a `DropZone`, the zone will hold a reference to the entity being dragged. It is up to the developer
to check if the source entity is acceptable and to indicate the drop action status.

> [!NOTE]
> `DropZone`s rely on `Interaction` to detect when something is over them.


### ResizeInteraction

Resize handles can be easily added to any widget, however these are just pre-styled draggables. They don't
automatically update the size of their container.

`PseudoState::Resizable(_)` states can be added to the outermost container to control which handle is interactible.


### UiContextRoot

The `UiContextRoot` is a marker component intended to signal _logical_ UI roots. The component is used by
context menus and tab container to find mounting points for spawned entites. In the case of context menus,
the menu itself will be mounted at this root. In the case of the tab container the floating panel that is
opened when a tab is "popped out" is mounted at the context root.

> [!TIP]
> Add a `UiContextRoot` to root parts of the UI that could be dynamically replaced, like screen roots.
> Switching screens then will clean up all context menus and floating panels that may have been dynamically
> spawned by the user without extra logic.


### UiUtils

`UiUtils` is a collection of useful UI logic, such as converting between variants of `Val` or finding UI viewports.


### Ui commands

A number of `Commands` extension is available to make life easier, such as managing `PseudoStates`,
`FluxInteractionStopwatchLock`s, or logging an entity's hierarchy and components.


### Context Menu

The context menu system in `sickle_ui` relies on reflection to allow any component to generate its own
entires to the context menu.

To implement a context menu for a widget two steps are necessary:
- An `Interaction` entity needs to be tagged with the `GenerateContextMenu` component to signal support.
- At least one component on the entity needs to implement `ContextMenuGenerator` and register its type.

```rust
impl Plugin for MyContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MyComponent>();
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, ContextMenuGenerator)]
pub struct MyComponent;


impl ContextMenuGenerator for MyComponent {
    fn build_context_menu(&self, context: Entity, container: &mut UiBuilder<ContextMenu>) {
        // Add menu items as needed to the container.
        
        container
            .menu_item(MenuItemConfig {
                name: "Open in new".into(),
                trailing_icon: icons.open_in_new,
                ..default()
            })
            // OpenInNewFromContextMenu is an examle component that another system could track
            // and execute the operation when the menu item is interacted.
            .insert(OpenInNewFromContextMenu { context });
    }

    fn placement_index(&self) -> usize {
        0
    }
}

```

The above example is the preparation for filling a context menu, but the menu item will only show up in
a context menu if the entity has `Interaction` and `GenerateContextMenu::default()` attached to it.

> [!NOTE]
> If the entity has multiple components with `ContextMenuGenerator` implementations, all of them will be
> used to generate the final context menu.

> [!TIP]
> Gaps in the placement indicies result in separators added to the menu.


### Locked style attributes

Style attributes can sometimes be locked. This is to prevent accidental styling of parts that have a
function and are driven by a widget's own system. Attempting to style or theme a locked attribute
results in a warning being emitted (and no change applied to the attribute).

> [!NOTE]
> Of course, changing `Style` (or any other style-related component) directly will bypass the checks.

> [!TIP]
> If a widget would still want to use the styling API while locking attributes, a `style_unchecked`
> line of commands is provided that skip the lock check.


### Tracked style state

Sometimes it is useful to track the overall state of styling application. `sickle_ui` provides a "meta"
style attribute called `TrackedStyleState` that can be used to drive other widgets from `DynamicStyle`s.
It does not actually style anything.

> [!NOTE]
> `TrackedStyleState` is `Lerp`able so it can be animated. Dropdowns use this to control the scroll view
> in their options panel to disable it while in transition to avoid visual pops.
