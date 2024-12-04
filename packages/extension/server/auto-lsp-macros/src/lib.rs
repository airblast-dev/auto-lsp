#![allow(deprecated)]

extern crate proc_macro;

use darling::FromDeriveInput;
use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, DataStruct, DeriveInput};

mod enum_builder;
mod feature_builder;
mod features;
mod meta;
mod paths;
mod struct_builder;
mod utilities;

use enum_builder::*;
use feature_builder::*;
use paths::*;
use struct_builder::*;

use crate::meta::*;
use crate::utilities::extract_fields::{match_enum_fields, match_struct_fields};

use std::cell::LazyCell;

trait BuildAstItem {
    fn generate_fields(&self) -> Vec<proc_macro2::TokenStream>;
    fn generate_ast_item_methods(&self) -> proc_macro2::TokenStream;
}

trait BuildAstItemBuilder {
    fn generate_builder_fields(&self) -> Vec<proc_macro2::TokenStream>;
    fn generate_builder_new(&self) -> proc_macro2::TokenStream;
    fn generate_query_binder(&self) -> proc_macro2::TokenStream;
    fn generate_add(&self) -> proc_macro2::TokenStream;
    fn generate_try_from(&self) -> proc_macro2::TokenStream;
}

const PATHS: LazyCell<Paths> = LazyCell::new(|| Paths::default());

#[proc_macro_attribute]
pub fn ast_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse args

    let attr_meta = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    let attributes = match UserFeatures::from_list(&attr_meta) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    // Parse input

    let input: DeriveInput = syn::parse_macro_input!(input);

    let derive_input = match StructInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    if !derive_input.data.is_struct() {
        return syn::Error::new_spanned(input, "Expected a struct")
            .to_compile_error()
            .into();
    }

    let input_name = &input.ident;
    let input_builder_name = format_ident!("{}Builder", input_name);

    let fields = match_struct_fields(&derive_input.data);
    let query_name = attributes.query_name;
    let mut tokens = proc_macro2::TokenStream::new();

    let input_attr = input.attrs;
    match attributes.kind {
        AstStructKind::Accessor => StructBuilder::new(
            None,
            &derive_input.data,
            &input_attr,
            &input_name,
            &input_builder_name,
            &query_name,
            &fields,
            &*PATHS,
            true,
        )
        .to_tokens(&mut tokens),
        AstStructKind::Symbol(symbol_features) => StructBuilder::new(
            Some(&symbol_features),
            &derive_input.data,
            &input_attr,
            &input_name,
            &input_builder_name,
            &query_name,
            &fields,
            &*PATHS,
            false,
        )
        .to_tokens(&mut tokens),
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn ast_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_name = &input.ident;
    let input_builder_name = format_ident!("{}Builder", input_name);
    let fields = match_enum_fields(&input.data);
    let mut tokens = proc_macro2::TokenStream::new();

    EnumBuilder::new(&input_name, &input_builder_name, &fields, &*PATHS).to_tokens(&mut tokens);
    tokens.into()
}
