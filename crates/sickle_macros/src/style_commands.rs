use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, Variant};

pub(crate) fn derive_style_commands_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    let Data::Enum(enum_data) = &ast.data else {
        return quote_spanned! {
            name_ident.span() => compile_error!("Invalid type: Must be an enum!");
        }
        .into();
    };

    let static_style_attribute = prepare_static_style_attribute(enum_data);

    quote! {
        #static_style_attribute
    }
    .into()
}

fn prepare_static_style_attribute(data: &DataEnum) -> proc_macro2::TokenStream {
    let static_variants: Vec<proc_macro2::TokenStream> =
        data.variants.iter().map(to_style_variant).collect();
    let static_eq_variants: Vec<proc_macro2::TokenStream> =
        data.variants.iter().map(to_eq_style_variant).collect();

    quote! {
        pub enum StaticStyleAttribute {
            #(#static_variants)*
            Custom(fn(Entity, &mut World)),
        }

        impl PartialEq for StaticStyleAttribute {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#static_eq_variants)*
                    (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
                    _ => false,
                }
            }
        }

        impl StaticStyleAttribute {
            pub fn apply(&self, ui_style: &mut UiStyle) {
                match self {
                    Self::BackgroundColor(value) => todo!(), //ui_style.background_color(value),
                    Self::Custom(callback) => {
                        ui_style.entity_commands().add(*callback);
                    }
                }
            }
        }
    }
}

fn to_style_variant(variant: &Variant) -> proc_macro2::TokenStream {
    eprintln!("{:?}", &variant.fields);
    let variant_ident = &variant.ident;
    quote! {
        #variant_ident(Color),
    }
}

fn to_eq_style_variant(variant: &Variant) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;
    quote! {
        (Self::#variant_ident(_), Self::#variant_ident(_)) => true,
    }
}
