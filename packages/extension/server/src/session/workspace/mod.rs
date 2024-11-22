use std::sync::{Arc, RwLock};

use auto_lsp::{macros::ast_builder::AstBuilder, traits::ast_item::AstItem};
use lsp_textdocument::FullTextDocument;
use lsp_types::Diagnostic;
use tree_sitter::Tree;
use tree_sitter_extend::builders::BuilderFn;

use super::cst_parser::CstParser;

pub mod add_document;
pub mod delete_document;
pub mod edit_document;
pub mod tree_sitter_extend;

pub struct Workspace {
    pub cst_parser: &'static CstParser,
    pub ast_builder: &'static BuilderFn,
    pub document: FullTextDocument,
    pub errors: Vec<Diagnostic>,
    pub cst: Tree,
    pub ast: Option<Arc<RwLock<dyn AstItem>>>,
}
