use lsp_types::{CompletionParams, CompletionResponse, CompletionTriggerKind};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor};

use crate::server::session::{Session, WORKSPACES};

impl Session {
    /// Get completion items for a document.
    ///
    /// TODO: Incomplete implementation
    pub fn get_completion_items(
        &mut self,
        params: CompletionParams,
    ) -> anyhow::Result<Option<CompletionResponse>> {
        let mut results = vec![];
        let uri = &params.text_document_position.text_document.uri;
        match params.context {
            Some(context) => match context.trigger_kind {
                CompletionTriggerKind::INVOKED => {
                    let workspace = WORKSPACES.lock();

                    let workspace = workspace
                        .get(uri)
                        .ok_or(anyhow::anyhow!("Workspace not found"))?;

                    let offset = workspace
                        .document
                        .descendant_at_position(params.text_document_position.position)
                        .unwrap()
                        .start_byte();

                    let source_code = workspace.document.document.text.as_str();
                    let content_bytes = source_code.as_bytes();

                    // Find the start of the word at the position
                    let mut start = offset;
                    while start > 0 && (content_bytes[start - 1] as char).is_alphanumeric() {
                        start -= 1;
                    }

                    // Find the end of the word at the position
                    let mut end = offset;
                    while end < content_bytes.len()
                        && (content_bytes[end] as char).is_alphanumeric()
                    {
                        end += 1;
                    }

                    let word = &source_code[start..end];

                    let query = Query::new(
                        &workspace.parsers.tree_sitter.language,
                        &format!("((identifier) @id (#match? @id \"^{}+\"))", word),
                    )
                    .unwrap();

                    let mut cursor = QueryCursor::new();
                    let mut captures = cursor.captures(
                        &query,
                        workspace.document.cst.root_node(),
                        workspace.document.document.text.as_bytes(),
                    );

                    while let Some((m, capture_index)) = captures.next() {
                        let capture = m.captures[*capture_index];

                        workspace
                            .ast
                            .iter()
                            .filter_map(|x| x.read().find_at_offset(capture.node.start_byte()))
                            .for_each(|x| {
                                x.read()
                                    .build_completion_items(&mut results, &workspace.document);
                            });
                    }
                }
                CompletionTriggerKind::TRIGGER_CHARACTER => {
                    // Todo
                }
                _ => {}
            },
            None => return Ok(None),
        };
        Ok(Some(results.into()))
    }
}
