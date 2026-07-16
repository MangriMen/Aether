extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

#[proc_macro_derive(RegisterSchema, attributes(schema_category, schema_name))]
pub fn derive_register_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Get category from attribute or default to "core"
    let category = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("schema_category"))
        .and_then(|attr| attr.parse_args::<LitStr>().ok())
        .map_or_else(|| "core".to_string(), |lit| lit.value());

    // Get custom schema name from attribute or fall back to type name
    let schema_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("schema_name"))
        .and_then(|attr| attr.parse_args::<LitStr>().ok())
        .map_or_else(|| name.to_string(), |lit| lit.value());

    let expanded = quote! {
        // Use full path to ensure it works regardless of imports
        ::register_schema::inventory::submit! {
            ::register_schema::SchemaEntry {
                name: #schema_name,
                category: #category,
                // Ensure schemars is available through the main crate
                schema: || ::register_schema::schemars::schema_for!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
