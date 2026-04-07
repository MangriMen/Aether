extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(RegisterSchema, attributes(schema_category))]
pub fn derive_register_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let category = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("schema_category"))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().ok())
        .map(|lit| lit.value())
        .unwrap_or_else(|| "core".to_string());

    let expanded = quote! {
        ::register_schema::inventory::submit! {
            ::register_schema::SchemaEntry {
                name: stringify!(#name),
                category: #category,
                schema: || ::schemars::schema_for!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
