extern crate proc_macro;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

use crate::feature_builder::FeaturesCodeGen;
use crate::field_builder::{FieldInfoExtract, Fields};
use crate::utilities::path_to_dot_tokens;
use crate::{ReferenceFeature, ReferenceFeatures, SymbolFeatures, PATHS};

use crate::Feature;

#[derive(Debug, FromMeta)]
pub struct SemanticTokenFeature {
    token_types: Path,
    token_type_index: String,
    range: Path,
    modifiers_fn: Option<Path>,
}

pub struct SemanticTokensBuilder<'a> {
    pub input_name: &'a Ident,
    pub fields: &'a Fields,
}

impl<'a> SemanticTokensBuilder<'a> {
    pub fn new(input_name: &'a Ident, fields: &'a Fields) -> Self {
        Self { input_name, fields }
    }

    pub fn default_impl(&self) -> TokenStream {
        let input_name = &self.input_name;
        let semantic_tokens_path = &PATHS.lsp_semantic_token.path;

        quote! {
            impl #semantic_tokens_path for #input_name { }
        }
    }
}

impl<'a> FeaturesCodeGen for SemanticTokensBuilder<'a> {
    fn code_gen(&self, params: &SymbolFeatures) -> impl quote::ToTokens {
        let input_name = &self.input_name;
        let semantic_tokens_path = &PATHS.lsp_semantic_token.path;
        let sig = &PATHS.lsp_semantic_token.build_semantic_tokens.sig;

        match &params.lsp_semantic_tokens {
            None => self.default_impl(),
            Some(params) => match params {
                Feature::User => quote! {},
                Feature::CodeGen(semantic) => {
                    let token_types = &semantic.token_types;
                    let token_index = &semantic.token_type_index;
                    let range = path_to_dot_tokens(&semantic.range, Some(quote! { read() }));

                    let field_names = &self.fields.field_names.get_field_names();
                    let field_vec_names = &self.fields.field_vec_names.get_field_names();
                    let field_option_names = &self.fields.field_option_names.get_field_names();

                    let modifiers = match &semantic.modifiers_fn {
                        None => quote! { 0 },
                        Some(path) => {
                            let path = path_to_dot_tokens(path, None);
                            quote! {
                                #path().iter().fold(0, |bitset, modifier| bitset | (1 << (*modifier)))
                            }
                        }
                    };

                    quote! {
                        impl #semantic_tokens_path for #input_name {
                            #sig {
                                let range = #range.get_lsp_range(doc);
                                match #token_types.get_index(#token_index) {
                                    Some(index) => builder.push(
                                        range,
                                        index as u32,
                                        #modifiers,
                                    ),
                                    None => {
                                        log::warn!("Warning: Token type not found {:?}", #token_index);
                                        return
                                    },
                                }
                                #(
                                    self.#field_names.read().build_semantic_tokens(doc, builder);
                                )*
                                #(
                                    if let Some(field) = self.#field_option_names.as_ref() {
                                        field.read().build_semantic_tokens(doc, builder);
                                    };
                                )*
                                #(
                                    for field in self.#field_vec_names.iter() {
                                        field.read().build_semantic_tokens(doc, builder);
                                    };
                                )*
                            }
                        }
                    }
                }
            },
        }
    }

    fn code_gen_reference(&self, params: &ReferenceFeatures) -> impl quote::ToTokens {
        let input_name = &self.input_name;
        let semantic_tokens_path = &PATHS.lsp_semantic_token.path;
        let sig = &PATHS.lsp_semantic_token.build_semantic_tokens.sig;

        match &params.lsp_semantic_tokens {
            None => self.default_impl(),
            Some(params) => match params {
                ReferenceFeature::Disable => self.default_impl(),
                ReferenceFeature::Reference => {
                    quote! {
                        impl #semantic_tokens_path for #input_name {
                            #sig {
                                if let Some(reference) = &self.get_target() {
                                    if let Some(reference) = reference.to_dyn() {
                                        reference.read().build_semantic_tokens(builder)
                                    }
                                }
                            }
                        }
                    }
                }
                ReferenceFeature::User => quote! {},
            },
        }
    }
}
