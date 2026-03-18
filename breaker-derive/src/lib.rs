//! Derive macro for generating Config structs from Defaults structs.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "proc macros require panic for error handling"
)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Meta, parse_macro_input};

/// Derives a `*Config` struct from a `*Defaults` struct.
///
/// Generates:
/// 1. A config struct with identical fields, deriving `Resource, Debug, Clone`
/// 2. `impl From<*Defaults> for *Config` (field-by-field copy)
/// 3. `impl Default for *Config` delegating to `Defaults::default().into()`
///
/// # Panics
///
/// Panics if the struct is missing `#[game_config(name = "...")]` or uses
/// unnamed fields.
///
/// # Usage
/// ```ignore
/// #[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
/// #[game_config(name = "BreakerConfig")]
/// pub struct BreakerDefaults { ... }
/// ```
#[proc_macro_derive(GameConfig, attributes(game_config))]
pub fn derive_game_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let defaults_name = &input.ident;

    // Extract config name from #[game_config(name = "...")] attribute
    let config_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if !attr.path().is_ident("game_config") {
                return None;
            }
            let Meta::List(meta_list) = &attr.meta else {
                return None;
            };
            let nested: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]> = meta_list
                .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                .ok()?;
            for meta in &nested {
                if let Meta::NameValue(nv) = meta
                    && nv.path.is_ident("name")
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                {
                    return Some(syn::Ident::new(&s.value(), s.span()));
                }
            }
            None
        })
        .expect("GameConfig derive requires #[game_config(name = \"...\")]");

    // Extract fields from struct data
    let syn::Data::Struct(data_struct) = &input.data else {
        panic!("GameConfig only supports structs");
    };
    let Fields::Named(fields) = &data_struct.fields else {
        panic!("GameConfig only supports structs with named fields");
    };

    let field_defs: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            let vis = &f.vis;
            let attrs: Vec<_> = f
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("doc"))
                .collect();
            quote! {
                #(#attrs)*
                #vis #name: #ty
            }
        })
        .collect();

    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

    let expanded = quote! {
        /// Configuration resource generated from [`#defaults_name`] by the `GameConfig` derive macro.
        #[derive(::bevy::prelude::Resource, Debug, Clone)]
        pub struct #config_name {
            #(#field_defs,)*
        }

        impl ::core::convert::From<#defaults_name> for #config_name {
            fn from(d: #defaults_name) -> Self {
                Self {
                    #(#field_names: d.#field_names,)*
                }
            }
        }

        impl ::core::default::Default for #config_name {
            fn default() -> Self {
                <#defaults_name as ::core::default::Default>::default().into()
            }
        }
    };

    TokenStream::from(expanded)
}
