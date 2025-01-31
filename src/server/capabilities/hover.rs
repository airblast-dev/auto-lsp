use lsp_types::{Hover, HoverParams};

use crate::server::session::{Session, WORKSPACES};

impl Session {
    /// Request to get hover information for a symbol at a position
    pub fn get_hover(&mut self, params: HoverParams) -> anyhow::Result<Option<Hover>> {
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
            Some(item) => Ok(item.read().get_hover(doc)),
            None => Ok(None),
        }
    }
}
