use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, AttrStyle, Attribute, Data, DataEnum, Fields, Meta, Type, TypePath, Variant,
};

#[derive(Clone, Copy, Debug)]
enum ParseError {
    InvalidVariant,
    NoFields,
    TooManyFields,
    InvalidType,
    InvalidTargetTuplType,
}

#[derive(Clone, Debug)]
struct StyleAttribute {
    ident: Ident,
    command: Ident,
    type_path: TypePath,
    target_tupl: Option<proc_macro2::TokenStream>,
    animatable: bool,
    target_enum: bool,
    skip_enity_command: bool,
    skip_ui_style_ext: bool,
    cmd_struct_name: String,
    cmd_struct_ident: Ident,
    target_attr_name: String,
}

impl StyleAttribute {
    fn new(ident: Ident, command: Ident, type_path: TypePath) -> Self {
        let cmd_struct_name = format!("Set{}", ident);
        let cmd_struct_ident = Ident::new(cmd_struct_name.as_str(), ident.span().clone());
        let target_attr_name = command.to_string();

        Self {
            ident,
            command,
            type_path,
            target_tupl: None,
            animatable: false,
            target_enum: false,
            skip_enity_command: false,
            skip_ui_style_ext: false,
            cmd_struct_name,
            cmd_struct_ident,
            target_attr_name,
        }
    }
}

pub(crate) fn derive_style_commands_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    let Data::Enum(enum_data) = &ast.data else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Invalid template type: Must be an enum!");
        }
        .into();
    };

    let attributes = match parse_variants(enum_data) {
        Ok(attributes) => attributes,
        Err((span, error)) => return match_error(span, error).into(),
    };

    let stylable_attribute = prepare_stylable_attribute(&attributes);
    let static_style_attribute = prepare_static_style_attribute(&attributes);
    let interactive_style_attribute = prepare_interactive_style_attribute(&attributes);
    let animated_style_attribute = prepare_animated_style_attribute(&attributes);
    let style_commands = prepare_style_commands(&attributes);

    quote! {
        #static_style_attribute
        #interactive_style_attribute
        #animated_style_attribute
        #stylable_attribute
        #style_commands
    }
    .into()
}

fn match_error(span: proc_macro2::Span, error: ParseError) -> proc_macro2::TokenStream {
    match error {
        ParseError::InvalidVariant => {
            return quote_spanned! {
                span => compile_error!("Invlaid variant: Must be a struct with named fields");
            }
        }
        ParseError::NoFields => {
            return quote_spanned! {
                span => compile_error!("No fields defined");
            }
        }
        ParseError::TooManyFields => {
            return quote_spanned! {
                span => compile_error!("Too many fields");
            }
        }
        ParseError::InvalidType => {
            return quote_spanned! {
                span => compile_error!("Invalid Type: Must be a TypePath");
            }
        }
        ParseError::InvalidTargetTuplType => {
            return quote_spanned! {
                span => compile_error!("Unsupported target_tupl value. Must be defined as #[target_tupl(Component)]");
            }
        }
    }
}

fn parse_variants(data: &DataEnum) -> Result<Vec<StyleAttribute>, (proc_macro2::Span, ParseError)> {
    let attributes: Result<Vec<_>, _> = data.variants.iter().map(parse_variant).collect();
    attributes
}

fn parse_variant(variant: &Variant) -> Result<StyleAttribute, (proc_macro2::Span, ParseError)> {
    let variant_ident = variant.ident.clone();

    let Fields::Named(fields) = variant.fields.clone() else {
        return Err((variant.span(), ParseError::InvalidVariant));
    };
    if fields.named.len() == 0 {
        return Err((variant.span(), ParseError::NoFields));
    }
    if fields.named.len() > 1 {
        return Err((variant.span(), ParseError::TooManyFields));
    }

    // Safe unwrap, we checked above that it extists
    let field = fields.named.first().unwrap();
    let Some(command) = field.ident.clone() else {
        return Err((field.ty.span(), ParseError::InvalidVariant));
    };

    let Type::Path(attr_type) = field.ty.clone() else {
        return Err((field.ty.span(), ParseError::InvalidType));
    };

    let mut attribute = StyleAttribute::new(variant_ident, command, attr_type);

    for attr in &variant.attrs {
        if attr.style == AttrStyle::Outer {
            if attr.path().is_ident("animatable") {
                attribute.animatable = true;
            } else if attr.path().is_ident("target_enum") {
                attribute.target_enum = true;
            } else if attr.path().is_ident("skip_enity_command") {
                attribute.skip_enity_command = true;
            } else if attr.path().is_ident("skip_ui_style_ext") {
                attribute.skip_ui_style_ext = true;
            } else if attr.path().is_ident("target_tupl") {
                let token_stream = target_tupl(attr)?;
                attribute.target_tupl = Some(token_stream);
            }
        }
    }

    Ok(attribute)
}

fn target_tupl(
    attr: &Attribute,
) -> Result<proc_macro2::TokenStream, (proc_macro2::Span, ParseError)> {
    let attr_span = attr.path().get_ident().unwrap().span();
    let Meta::List(list) = &attr.meta else {
        return Err((attr_span, ParseError::InvalidTargetTuplType));
    };

    if list.tokens.is_empty() {
        return Err((attr_span, ParseError::InvalidTargetTuplType));
    }

    let tokens = list.tokens.clone().into_iter();
    let has_invalid_parts = tokens.clone().any(|e| match e {
        proc_macro2::TokenTree::Group(_) => true,
        proc_macro2::TokenTree::Ident(_) => false,
        proc_macro2::TokenTree::Punct(_) => false,
        proc_macro2::TokenTree::Literal(_) => true,
    });

    if tokens.clone().count() == 0 || has_invalid_parts {
        return Err((attr_span, ParseError::InvalidTargetTuplType));
    }

    Ok(list.tokens.clone())
}

fn prepare_stylable_attribute(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream {
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_style_attribute_variant)
        .collect();

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
        pub enum StylableAttribute {
            #(#base_variants)*
        }
    }
}

fn prepare_static_style_attribute(
    style_attributes: &Vec<StyleAttribute>,
) -> proc_macro2::TokenStream {
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_static_style_variant)
        .collect();
    let eq_variants: Vec<proc_macro2::TokenStream> =
        style_attributes.iter().map(to_eq_style_variant).collect();
    let apply_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_static_style_apply_variant)
        .collect();

    quote! {
        pub enum StaticStyleAttribute {
            #(#base_variants)*
            Custom(fn(Entity, &mut World)),
        }

        impl PartialEq for StaticStyleAttribute {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#eq_variants)*
                    (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
                    _ => false,
                }
            }
        }

        impl StaticStyleAttribute {
            pub fn apply<'a>(self, ui_style: &'a mut UiStyle<'a>) {
                match self {
                    #(#apply_variants)*
                    Self::Custom(callback) => {
                        ui_style.entity_commands().add(callback);
                    }
                }
            }
        }
    }
}

fn prepare_interactive_style_attribute(
    style_attributes: &Vec<StyleAttribute>,
) -> proc_macro2::TokenStream {
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_interactive_style_variant)
        .collect();
    let eq_variants: Vec<proc_macro2::TokenStream> =
        style_attributes.iter().map(to_eq_style_variant).collect();
    let apply_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .map(to_interactive_style_appl_variant)
        .collect();

    quote! {
        pub enum InteractiveStyleAttribute {
            #(#base_variants)*
            Custom(fn(Entity, FluxInteraction, &mut World)),
        }

        impl PartialEq for InteractiveStyleAttribute {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#eq_variants)*
                    (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
                    _ => false,
                }
            }
        }

        impl InteractiveStyleAttribute {
            fn to_attribute(&self, flux_interaction: FluxInteraction) -> StaticStyleAttribute {
                match self {
                    #(#apply_variants)*
                    Self::Custom(_) => unreachable!(),
                }
            }

            pub fn apply<'a>(&self, flux_interaction: FluxInteraction, ui_style: &'a mut UiStyle<'a>) {
                match self {
                    Self::Custom(callback) => {
                        ui_style
                            .entity_commands()
                            .add(CustomInteractiveStyleAttribute {
                                callback: *callback,
                                flux_interaction,
                            });
                    }
                    _ => {
                        self.to_attribute(flux_interaction).apply(ui_style);
                    }
                }
            }
        }
    }
}

fn prepare_animated_style_attribute(
    style_attributes: &Vec<StyleAttribute>,
) -> proc_macro2::TokenStream {
    let base_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| v.animatable)
        .map(to_animated_style_variant)
        .collect();
    let eq_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| v.animatable)
        .map(to_eq_style_variant)
        .collect();
    let apply_variants: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| v.animatable)
        .map(to_animated_style_appl_variant)
        .collect();

    quote! {
        pub enum AnimatedStyleAttribute {
            #(#base_variants)*
            Custom(fn(Entity, InteractionAnimationState, InteractionAnimationState, &mut World)),
        }

        impl PartialEq for AnimatedStyleAttribute {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#eq_variants)*
                    (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
                    _ => false,
                }
            }
        }

        impl AnimatedStyleAttribute {
            fn to_attribute(
                &self,
                transition_base: InteractionAnimationState,
                animation_progress: InteractionAnimationState,
            ) -> StaticStyleAttribute {
                match self {
                    #(#apply_variants)*
                    Self::Custom(_) => unreachable!(),
                }
            }

            pub fn apply<'a>(
                &self,
                transition_base: InteractionAnimationState,
                animation_progress: InteractionAnimationState,
                ui_style: &'a mut UiStyle<'a>,
            ) {
                match self {
                    Self::Custom(callback) => {
                        ui_style
                            .entity_commands()
                            .add(CustomAnimatableStyleAttribute {
                                callback: *callback,
                                transition_base,
                                animation_progress,
                            });
                    }
                    _ => {
                        self
                            .to_attribute(transition_base, animation_progress)
                            .apply(ui_style);
                    }
                }
            }
        }
    }
}

fn prepare_style_commands(style_attributes: &Vec<StyleAttribute>) -> proc_macro2::TokenStream {
    let extensions: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| !v.skip_ui_style_ext)
        .map(to_ui_style_extensions)
        .collect();

    let implementations: Vec<proc_macro2::TokenStream> = style_attributes
        .iter()
        .filter(|v| !(v.skip_ui_style_ext || v.skip_enity_command))
        .map(to_ui_style_command_impl)
        .collect();

    quote! {
        #(#extensions)*
        #(#implementations)*
    }
}

fn to_eq_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    quote! {
        (Self::#ident(_), Self::#ident(_)) => true,
    }
}

fn to_style_attribute_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    quote! {
        #ident,
    }
}

fn to_static_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    let type_path = &style_attribute.type_path;
    quote! {
        #ident(#type_path),
    }
}

fn to_interactive_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    let type_path = &style_attribute.type_path;
    quote! {
        #ident(StaticValueBundle<#type_path>),
    }
}

fn to_animated_style_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    let type_path = &style_attribute.type_path;
    quote! {
        #ident(AnimatedValueBundle<#type_path>),
    }
}

fn to_static_style_apply_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    let command = &style_attribute.command;
    quote! {
        Self::#ident(value) => {
            ui_style.#command(value);
        }
    }
}

fn to_interactive_style_appl_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    quote! {
        Self::#ident(bundle) => {
            StaticStyleAttribute::#ident(bundle.to_value(flux_interaction))
        },
    }
}

fn to_animated_style_appl_variant(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    quote! {
        Self::#ident(bundle) => StaticStyleAttribute::#ident(
            bundle.to_value(transition_base, animation_progress),
        ),
    }
}

fn to_ui_style_extensions(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let cmd_struct_name = &style_attribute.cmd_struct_name.clone();
    let cmd_struct_ident = &style_attribute.cmd_struct_ident.clone();
    let target_attr = &style_attribute.command;
    let target_type = &style_attribute.type_path;

    let extension_name = String::from(cmd_struct_name.clone()) + "Ext";
    let extension_ident = Ident::new(extension_name.as_str(), cmd_struct_ident.span().clone());
    let extension_unchecked_name = String::from(cmd_struct_name.as_str()) + "UncheckedExt";
    let extension_unchecked_ident = Ident::new(
        extension_unchecked_name.as_str(),
        cmd_struct_ident.span().clone(),
    );

    quote! {
        struct #cmd_struct_ident {
            #target_attr: #target_type,
            check_lock: bool,
        }

        pub trait #extension_ident<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyle<'a>;
        }

        impl<'a> #extension_ident<'a> for UiStyle<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyle<'a> {
                self.entity_commands().add(#cmd_struct_ident {
                    #target_attr,
                    check_lock: true
                });
                self
            }
        }

        pub trait #extension_unchecked_ident<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyleUnchecked<'a>;
        }

        impl<'a> #extension_unchecked_ident<'a> for UiStyleUnchecked<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyleUnchecked<'a> {
                self.entity_commands().add(#cmd_struct_ident {
                    #target_attr,
                    check_lock: false
                });
                self
            }
        }
    }
}

fn to_ui_style_command_impl(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let ident = &style_attribute.ident;
    let cmd_struct_ident = &style_attribute.cmd_struct_ident.clone();
    let target_attr_name = &style_attribute.target_attr_name;
    let setter = to_setter_entity_command_frag(style_attribute);

    quote! {
        impl EntityCommand for #cmd_struct_ident {
            fn apply(self, entity: Entity, world: &mut World) {
                if self.check_lock {
                    if let Some(locked_attrs) = world.get::<LockedStyleAttributes>(entity) {
                        if locked_attrs.contains(StylableAttribute::#ident) {
                            warn!(
                                "Failed to style {} property on entity {:?}: Attribute locked!",
                                #target_attr_name,
                                entity
                            );
                            return;
                        }
                    }
                }

                #setter
            }
        }
    }
}

fn to_setter_entity_command_frag(style_attribute: &StyleAttribute) -> proc_macro2::TokenStream {
    let target_attr = &style_attribute.command;
    let target_type = &style_attribute.type_path;
    let target_attr_name = &style_attribute.target_attr_name;

    if style_attribute.target_enum {
        let target_type_name = target_type.path.get_ident().unwrap().to_string();

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#target_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No {} component found!",
                    #target_attr_name,
                    entity,
                    #target_type_name
                );
                return;
            };

            if *#target_attr != self.#target_attr {
                *#target_attr = self.#target_attr;
            }
        }
    } else if let Some(target_tupl) = &style_attribute.target_tupl {
        let component_type = target_tupl.clone();
        let component_name: Vec<String> = target_tupl
            .clone()
            .into_iter()
            .map(|tt| tt.to_string())
            .collect();
        let component_name = component_name.join("");

        quote! {
            let Some(mut #target_attr) = world.get_mut::<#component_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No {} component found!",
                    #target_attr_name,
                    entity,
                    #component_name,
                );
                return;
            };

            if #target_attr.0 != self.#target_attr {
                #target_attr.0 = self.#target_attr;
            }
        }
    } else {
        quote! {
            let Some(mut style) = world.get_mut::<Style>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No Style component found!",
                    #target_attr_name,
                    entity
                );
                return;
            };

            if style.#target_attr != self.#target_attr {
                style.#target_attr = self.#target_attr;
            }
        }
    }
}

// TODO: Add skip_styleable_enum (so Position cannot be locked) attr or skip_lock_variant attr
// TODO: Add static-only attr (FluxInteraction cannot be interactive :D)
