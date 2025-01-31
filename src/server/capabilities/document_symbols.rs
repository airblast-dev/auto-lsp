use auto_lsp_core::ast::VecOrSymbol;
use lsp_types::{DocumentSymbolParams, DocumentSymbolResponse};

use crate::server::session::{Session, WORKSPACES};

impl Session {
    /// Request to get document symbols for a file
    ///
    /// This function will recursively traverse the ast and return all symbols found.
    pub fn get_document_symbols(
        &mut self,
        params: DocumentSymbolParams,
    ) -> anyhow::Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let workspace = WORKSPACES.lock();

        let workspace = workspace
            .get(&uri)
            .ok_or(anyhow::anyhow!("Workspace not found"))?;

        let source = &workspace.document;

        let symbols = workspace
            .ast
            .iter()
            .filter_map(|p| p.read().get_document_symbols(source))
            .collect::<Vec<_>>();

        Ok(Some(DocumentSymbolResponse::Nested(
            symbols
                .into_iter()
                .flat_map(|s| match s {
                    VecOrSymbol::Symbol(s) => vec![s],
                    VecOrSymbol::Vec(v) => v,
                })
                .collect(),
        )))
    }
}
