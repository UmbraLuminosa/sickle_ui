use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::{quote, quote_spanned};
use syn::{self, AttrStyle, Data::Struct, Fields::Named, Meta, Path, Type};

fn parse_lock_attribute(
    ast: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let Some(lock_attr) = ast
        .attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("lock_attr"))
    {
        let error = quote_spanned! {
            lock_attr.path().get_ident().unwrap().span() => compile_error!( "Unsupported lock_attr value. Must be defined as #[lock_attr(StylableAttribute::VARIANT)]");
        };

        let Meta::List(list) = &lock_attr.meta else {
            return Err(error.into());
        };

        if list.tokens.is_empty() {
            return Err(error.into());
        }

        let mut tokens = list.tokens.clone().into_iter();
        if tokens.clone().count() != 4 {
            return Err(error.into());
        }

        if let TokenTree::Ident(enum_name) = &tokens.next().unwrap() {
            if *enum_name != Ident::new("StylableAttribute", enum_name.span().clone()) {
                return Err(error.into());
            }

            let stylable_attr = list.tokens.clone();
            return Ok(quote! {
                if let Some(locked_attrs) = world.get::<LockedStyleAttributes>(entity){
                    if locked_attrs.contains(#stylable_attr){
                        warn!(
                            "Failed to style {:?} property on entity {:?}: Attribute locked!",
                            #stylable_attr,
                            entity
                        );
                        return;
                    }
                }
            });
        } else {
            return Err(error.into());
        }
    } else {
        Ok(proc_macro2::TokenStream::new())
    }
}

fn parse_target_setter(
    ast: &syn::DeriveInput,
    target_attr: proc_macro2::Ident,
    target_name: String,
    target_type: Path,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let Some(target_enum) = ast
        .attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("target_enum"))
    {
        let error = quote_spanned! {
            target_enum.path().get_ident().unwrap().span() => compile_error!( "Unsupported target_enum value. Must be defined as #[target_enum]");
        };

        let Meta::Path(_) = &target_enum.meta else {
            return Err(error.into());
        };

        let target_type_name = target_type.get_ident().unwrap().to_string();

        Ok(quote! {
            let Some(mut #target_attr) = world.get_mut::<#target_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No {} component found!",
                    #target_name,
                    entity,
                    #target_type_name
                );
                return;
            };

            if *#target_attr != self.#target_attr {
                *#target_attr = self.#target_attr;
            }
        })
    } else if let Some(target_tupl) = ast
        .attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("target_tupl"))
    {
        let error = quote_spanned! {
            target_tupl.path().get_ident().unwrap().span() => compile_error!( "Unsupported target_tupl value. Must be defined as #[target_tupl(Component)]");
        };

        let Meta::List(list) = &target_tupl.meta else {
            return Err(error.into());
        };

        if list.tokens.is_empty() {
            return Err(error.into());
        }

        let tokens = list.tokens.clone().into_iter();
        if tokens.clone().count() == 0 {
            return Err(error.into());
        }

        let component_type = list.tokens.clone();
        let component_name: Vec<String> = tokens.map(|tt| tt.to_string()).collect();
        let component_name = component_name.join("");

        Ok(quote! {
            let Some(mut #target_attr) = world.get_mut::<#component_type>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No {} component found!",
                    #target_name,
                    entity,
                    #component_name,
                );
                return;
            };

            if #target_attr.0 != self.#target_attr {
                #target_attr.0 = self.#target_attr;
            }
        })
    } else {
        Ok(quote! {
            let Some(mut style) = world.get_mut::<Style>(entity) else {
                warn!(
                    "Failed to set {} property on entity {:?}: No Style component found!",
                    #target_name,
                    entity
                );
                return;
            };

            if style.#target_attr != self.#target_attr {
                style.#target_attr = self.#target_attr;
            }
        })
    }
}

pub(crate) fn derive_style_command_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    let name = &ast.ident.to_string();
    let name_unchecked = String::from(name) + "Unchecked";
    let name_unchecked_ident = Ident::new(name_unchecked.as_str(), name_ident.span().clone());
    let extension_name = String::from(name) + "Ext";
    let extension_ident = Ident::new(extension_name.as_str(), name_ident.span().clone());
    let extension_unchecked_name = String::from(name_unchecked) + "Ext";
    let extension_unchecked_ident =
        Ident::new(extension_unchecked_name.as_str(), name_ident.span().clone());

    let Struct(struct_data) = &ast.data else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Unsupported Data type, only Structs with named fields are supported");
        }.into();
    };

    let Named(named_fields) = &struct_data.fields else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Unsupported Struct type, only Structs with named fields are supported");
        }
        .into();
    };

    if named_fields.named.iter().count() != 1 {
        return quote_spanned! {
            name_ident.span() => compile_error!("Command Struct must have exactly one field: {target_attr: TargetType}");
        }
        .into();
    }

    let target_field = named_fields.named.iter().next().unwrap();
    let target_attr = target_field.ident.clone().unwrap();
    let target_name = target_attr.to_string();
    let Type::Path(target_path) = &target_field.ty else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Cannot find value field", #name);
        }
        .into();
    };
    let target_type = target_path.path.clone();

    let stylable_attr_check = match parse_lock_attribute(ast) {
        Ok(stream) => stream,
        Err(error) => return error.into(),
    };

    let value_setter = match parse_target_setter(
        ast,
        target_attr.clone(),
        target_name.clone(),
        target_type.clone(),
    ) {
        Ok(setter) => setter,
        Err(error) => return error.into(),
    };

    quote! {
        impl EntityCommand for #name_ident {
            fn apply(self, entity: Entity, world: &mut World) {
                #stylable_attr_check
                #value_setter
            }
        }

        pub trait #extension_ident<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyle<'a>;
        }

        impl<'a> #extension_ident<'a> for UiStyle<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyle<'a> {
                self.commands.add(#name_ident {
                    #target_attr,
                });
                self
            }
        }

        struct #name_unchecked_ident {
            #target_attr: #target_type
        }

        impl EntityCommand for #name_unchecked_ident {
            fn apply(self, entity: Entity, world: &mut World) {
                #value_setter
            }
        }

        pub trait #extension_unchecked_ident<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyleUnchecked<'a>;
        }

        impl<'a> #extension_unchecked_ident<'a> for UiStyleUnchecked<'a> {
            fn #target_attr(&'a mut self, #target_attr: #target_type) -> &mut UiStyleUnchecked<'a> {
                self.commands.add(#name_unchecked_ident {
                    #target_attr,
                });
                self
            }
        }
    }
    .into()
}
