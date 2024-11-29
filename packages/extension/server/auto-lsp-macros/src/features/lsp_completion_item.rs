extern crate proc_macro;

use crate::{
    utilities::{extract_fields::StructFields, format_tokens::path_to_dot_tokens}, FeaturesCodeGen, Paths, ToCodeGen
};
use darling::FromMeta;
use quote::quote;
use syn::{Ident, Path};

use crate::Feature;

#[derive(Debug, FromMeta)]
pub struct CompletionItemFeature {
    item: CompletionItem,
}

#[derive(Debug, FromMeta)]
pub struct CompletionItem {
    label: Path,
    kind: Path,
}

pub struct CompletionItemsBuilder<'a> {
    pub input_name: &'a Ident,
    pub paths: &'a Paths,
    pub params: Option<&'a Feature<CompletionItemFeature>>,
    pub fields: &'a StructFields,
    pub is_accessor: bool
}

impl<'a> CompletionItemsBuilder<'a> {
    pub fn new(
        input_name: &'a Ident,
        paths: &'a Paths,
        params: Option<&'a Feature<CompletionItemFeature>>,
        fields: &'a StructFields,
        is_accessor: bool,
    ) -> Self {
        Self {
            input_name,
            params,
            paths,
            fields,
            is_accessor,
        }
    }
}

impl<'a> ToCodeGen for CompletionItemsBuilder<'a> {
    fn to_code_gen(&self, codegen: &mut FeaturesCodeGen) {
        let input_name = &self.input_name;
        let completion_items_path = &self.paths.completion_items_trait;

        if self.is_accessor {
            codegen.input.other_impl.push(quote! {
                impl #completion_items_path for #input_name {
                    fn build_completion_items(&self, acc: &mut Vec<lsp_types::CompletionItem>, doc: &lsp_textdocument::FullTextDocument) {
                        if let Some(accessor) = &self.accessor {
                            if let Some(accessor) = accessor.to_dyn() {
                                accessor.read().build_completion_items(acc, doc)
                            }
                        }                        
                    }
                }
            });
            return
        } 

        match self.params {
            None => codegen.input.other_impl.push(quote! {
                impl #completion_items_path for #input_name {
                    fn build_completion_items(
                        &self,
                        _acc: &mut Vec<lsp_types::CompletionItem>,
                        _doc: &lsp_textdocument::FullTextDocument,
                    ) {}
                }
            }),
            Some(params) => match params {
                Feature::User => (),
                Feature::CodeGen(completion) => {
                    let item = &completion.item;
                    let kind = &item.kind;
                    let label = path_to_dot_tokens(&item.label, None);
                
                    codegen.input.impl_base.push(quote! {
                        const LSP_COMPLETION_ITEM_KIND: &'static lsp_types::CompletionItemKind = &#kind;
                    });
                
                    codegen.input.other_impl.push(quote! {
                        impl #completion_items_path for #input_name {
                            fn build_completion_items(&self, acc: &mut Vec<lsp_types::CompletionItem>, doc: &lsp_textdocument::FullTextDocument) {
                                let read = #label.read();
                
                                acc.push(lsp_types::CompletionItem {
                                    label: read.get_text(doc.get_content(None).as_bytes()).to_string(),
                                    kind: Some(*Self::LSP_COMPLETION_ITEM_KIND),
                                    detail: None,
                                    ..Default::default()
                                });
                            }
                        }
                    })                      
                }
            }
        }
    }
}
