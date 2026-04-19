extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

#[proc_macro_derive(RegisterSchema, attributes(schema_category))]
pub fn derive_register_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Get category from attribute or default to "core"
    let category = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("schema_category"))
        .and_then(|attr| {
            // Support #[schema_category("name")]
            attr.parse_args::<LitStr>().ok()
        })
        .map_or_else(|| "core".to_string(), |lit| lit.value());

    let expanded = quote! {
        // Use full path to ensure it works regardless of imports
        ::register_schema::inventory::submit! {
            ::register_schema::SchemaEntry {
                name: stringify!(#name),
                category: #category,
                // Ensure schemars is available through the main crate
                schema: || ::register_schema::schemars::schema_for!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
