use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn derive_ui_context_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name_ident = &ast.ident;
    let ident_name = name_ident.to_string();
    quote! {
        impl UiContext for #name_ident { }
    }
    .into()
}
