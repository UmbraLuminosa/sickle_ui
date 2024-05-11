use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn derive_default_theme_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    quote! {
        impl DefaultTheme for #name_ident {}
    }
    .into()
}
