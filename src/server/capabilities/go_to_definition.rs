use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use crate::server::session::{Session, WORKSPACES};

impl Session {
    /// Request to go to the definition of a symbol
    ///
    /// The trait [`crate::core::ast::GetGoToDefinition`] needs to be implemented otherwise this will return None.
    pub fn go_to_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> anyhow::Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let workspace = WORKSPACES.lock();

        let workspace = workspace
            .get(&uri)
            .ok_or(anyhow::anyhow!("Workspace not found"))?;

        let position = params.text_document_position_params.position;
        let doc = &workspace.document;

        let offset = doc.offset_at(position).unwrap();
        let item = workspace
            .ast
            .iter()
            .find_map(|symbol| symbol.read().find_at_offset(offset));

        match item {
            Some(item) => Ok(item.read().go_to_definition(doc)),
            None => Ok(None),
        }
    }
}
