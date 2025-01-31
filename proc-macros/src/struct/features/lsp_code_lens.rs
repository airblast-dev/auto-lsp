extern crate proc_macro;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

use crate::field_builder::{FieldInfoExtract, Fields};
use crate::utilities::path_to_dot_tokens;
use crate::{
    r#struct::feature_builder::FeaturesCodeGen, ReferenceFeature, ReferenceFeatures,
    SymbolFeatures, PATHS,
};

use crate::Feature;

#[derive(Debug, FromMeta)]
pub struct CodeLensFeature {
    pub code_lens_fn: Path,
}

pub struct CodeLensBuilder<'a> {
    pub input_name: &'a Ident,
    pub fields: &'a Fields,
}

impl<'a> CodeLensBuilder<'a> {
    pub fn new(input_name: &'a Ident, fields: &'a Fields) -> Self {
        Self { input_name, fields }
    }

    pub fn default_impl(&self) -> TokenStream {
        let input_name = &self.input_name;
        let code_lens_path = &PATHS.lsp_code_lens.path;

        quote! {
            impl #code_lens_path for #input_name { }
        }
    }
}

impl<'a> FeaturesCodeGen for CodeLensBuilder<'a> {
    fn code_gen(&self, params: &SymbolFeatures) -> impl quote::ToTokens {
        let input_name = &self.input_name;
        let code_lens_path = &PATHS.lsp_code_lens.path;
        let sig = &PATHS.lsp_code_lens.build_code_lens.sig;

        match &params.lsp_code_lens {
            None => self.default_impl(),
            Some(params) => match params {
                Feature::User => quote! {},
                Feature::CodeGen(code_lens) => {
                    let call = path_to_dot_tokens(&code_lens.code_lens_fn, None);

                    let field_names = &self.fields.field_names.get_field_names();
                    let field_vec_names = &self.fields.field_vec_names.get_field_names();
                    let field_option_names = &self.fields.field_option_names.get_field_names();

                    quote! {
                        impl #code_lens_path for #input_name {
                            #sig {
                                #call(acc);
                                #(
                                    self.#field_names.read().build_code_lens(acc);
                                )*
                                #(
                                    if let Some(field) = self.#field_option_names.as_ref() {
                                        field.read().build_code_lens(acc);
                                    };
                                )*
                                #(
                                    for field in self.#field_vec_names.iter() {
                                        field.read().build_code_lens(acc);
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
        let code_lens_path = &PATHS.lsp_code_lens.path;
        let sig = &PATHS.lsp_code_lens.build_code_lens.sig;

        match &params.lsp_code_lens {
            None => self.default_impl(),
            Some(feature) => match feature {
                ReferenceFeature::Disable => self.default_impl(),
                ReferenceFeature::Reference => {
                    quote! {
                        impl #code_lens_path for #input_name {
                            #sig {
                                if let Some(reference) = &self.get_target() {
                                    if let Some(reference) = reference.to_dyn() {
                                        return reference.read().build_code_lens(acc)
                                    }
                                }
                                None
                            }
                        }
                    }
                }
                ReferenceFeature::User => quote! {},
            },
        }
    }
}
