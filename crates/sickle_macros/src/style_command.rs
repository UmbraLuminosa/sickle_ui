use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{self, Data::Struct, Fields::Named, Type};

pub(crate) fn derive_style_command_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    let name = &ast.ident.to_string();
    let extension_name = String::from(name) + "Ext";
    let extension_ident = Ident::new(extension_name.as_str(), name_ident.span().clone());

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

    quote! {
        impl EntityCommand for #name_ident {
            fn apply(self, entity: Entity, world: &mut World) {
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
    }
    .into()
}
