extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

/// The string argument of `#[attribute("...")]`; an attribute whose argument is not a string
/// literal is an error, not a missing one.
fn string_attribute(input: &DeriveInput, attribute: &str) -> syn::Result<Option<String>> {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(attribute))
        .map(|attr| attr.parse_args::<LitStr>().map(|lit| lit.value()))
        .transpose()
}

#[proc_macro_derive(RegisterSchema, attributes(schema_category, schema_name))]
pub fn derive_register_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Get category from attribute or default to "core"
    let category = match string_attribute(&input, "schema_category") {
        Ok(category) => category.unwrap_or_else(|| "core".to_string()),
        Err(error) => return error.to_compile_error().into(),
    };

    // Get custom schema name from attribute or fall back to type name
    let schema_name = match string_attribute(&input, "schema_name") {
        Ok(schema_name) => schema_name.unwrap_or_else(|| name.to_string()),
        Err(error) => return error.to_compile_error().into(),
    };

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
