mod event_handler;
mod simple_interaction_plugin;
mod style_command;
mod style_commands;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// Macro to derive an EntityCommand to register and store one-shot systems.
///
/// # Dependencies
/// ```
/// [lib]
/// proc-macro = true
///
/// [dependencies]
/// syn = "1.0"
/// quote = "1.0"
/// proc-macro2 = "1.0"
/// ```
///
/// See: https://doc.rust-lang.org/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes
///
/// # Usage
///
/// See it in action: https://youtu.be/s1lQD-R_kqg
///
/// ```
/// #[derive(Component, EventHandler)]
/// pub struct OnSomething {
///   system_id: SystemId,
///   pub active: bool,
/// }
///
/// impl Plugin for SomethingPlugin {
///   fn build(&self, app: &mut App) {
///       app.add_systems(Update, handle_pressed);
///   }
/// }
///
/// // Call this system when you need to do something
/// fn handle_pressed(
///     mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<OnSomething>)>,
///     mut handler_query: Query<&mut OnSomething>,
///     mut commands: Commands,
/// ) {
///     for (entity, interaction) in &mut interaction_query {
///         let handler = handler_query.get_mut(entity).unwrap().into_inner();
///         if *interaction == Interaction::Pressed {
///             let mut active_handler = handler.clone();
///             active_handler.active = true;
///             commands.entity(entity).insert(active_handler);
///             commands.run_system(handler.system_id);
///             commands.entity(entity).insert(handler.clone());
///         }
///     }
/// }
///
/// // Spawn a UI button from e.g. a Startup system. `MainMenuRootNode` would be a marker component
/// // somewhere in the app, used to mark a container.
/// fn spawn_something_button(mut commands: Commands, root_node: Query<Entity, With<MainMenuRootNode>>) {
///     commands.entity(root_node.single()).with_children(|parent| {
///         parent
///             .spawn(ButtonBundle {
///                 style: Style {
///                     width: Val::Px(200.),
///                     border: UiRect::all(Val::Px(2.)),
///                     ..default()
///                 },
///                 background_color: Color::rgb(0.35, 0.35, 0.35).into(),
///                 ..default()
///             })
///             // A component can only be added once, but the handler can be any system
///             //.add(OnSomethingHandler::from(||println!("Button pressed")))
///             //.add(OnSomethingHandler::from(do_the_something_general))
///             .add(OnSomethingHandler::from(do_the_something_with_the_button));
///     });
/// }
///
/// fn do_the_something_general(){
///     // Do something without caring which button is pressed
/// }
///
/// // Do something for the exact button being pressed
/// fn do_the_something_with_the_button(
///   mut pressed_query: Query<(Entity, &OnSomething), With<OnSomething>>,
/// ) {
///   for (entity, on_something) in &mut pressed_query {
///     if on_something.active {
///       // Do something!
///     }
///   }
/// }
///
/// ```
///
#[proc_macro_derive(EventHandler)]
pub fn event_handler_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    event_handler::derive_event_handler_macro(&ast)
}

#[proc_macro_attribute]
pub fn simple_interaction_for(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as DeriveInput);
    simple_interaction_plugin::impl_simple_interaction_plugin_macro(attr, ast.ident)
}

#[proc_macro_derive(StyleCommand, attributes(lock_attr, target_enum, target_tupl))]
pub fn style_command_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    style_command::derive_style_command_macro(&ast)
}

#[proc_macro_derive(
    StyleCommands,
    attributes(
        static_style_only,
        skip_enity_command,
        skip_ui_style_ext,
        skip_lockable_enum,
        animatable,
        target_enum,
        target_tupl
    )
)]
pub fn style_commands_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();
    style_commands::derive_style_commands_macro(&ast)
}
