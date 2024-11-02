use crate::{
    features::{borrowable::BorrowableFeature, lsp_inlay_hint::InlayHintFeature},
    DocumentSymbolFeature, HoverFeature, SemanticTokenFeature,
};
use darling::FromMeta;
use syn::Path;

#[derive(Debug, FromMeta)]
pub struct SymbolArgs {
    pub query_name: String,
    pub features: Option<Features>,
}

#[derive(Debug, FromMeta)]
pub struct Features {
    pub borrowable: Option<BorrowableFeature>,
    // LSP
    pub lsp_document_symbols: Option<DocumentSymbolFeature>,
    pub lsp_hover: Option<HoverFeature>,
    pub lsp_semantic_token: Option<SemanticTokenFeature>,
    pub lsp_inlay_hint: Option<InlayHintFeature>,
}
