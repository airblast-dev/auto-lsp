use crate::core_ast::data::ReferrersTrait;
use crate::core_ast::{symbol::*, update::UpdateRange};
use crate::workspace::Document;
use lsp_types::{Diagnostic, Url};
use std::{ops::ControlFlow, sync::Arc};
use tree_sitter::InputEdit;

pub struct MainBuilder<'a> {
    pub query: &'a tree_sitter::Query,
    pub document: &'a Document,
    pub url: Arc<Url>,
    pub diagnostics: &'a mut Vec<Diagnostic>,
    pub unsolved_checks: &'a mut Vec<WeakSymbol>,
    pub unsolved_references: &'a mut Vec<WeakSymbol>,
}

impl<'a> MainBuilder<'a> {
    #[cfg(not(feature = "rayon"))]
    pub fn resolve_references(&mut self) -> &mut Self {
        self.unsolved_references.retain(|item| {
            let item = match item.to_dyn() {
                Some(read) => read,
                None => return false,
            };
            let read = item.read();
            match read.find(&self.document) {
                Ok(Some(target)) => {
                    target.write().add_referrer(item.to_weak());
                    drop(read);
                    item.write().set_target_reference(target.to_weak());
                    false
                }
                Ok(None) => true,
                Err(err) => {
                    self.diagnostics.push(err);
                    true
                }
            }
        });
        self
    }

    #[cfg(feature = "rayon")]
    pub fn resolve_references(&mut self) -> &mut Self {
        use parking_lot::RwLock;
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

        let diagnostics = RwLock::new(vec![]);
        *self.unsolved_references = self
            .unsolved_references
            .par_iter()
            .cloned()
            .filter(|item| {
                let item = match item.to_dyn() {
                    Some(read) => read,
                    None => return false,
                };
                let read = item.read();
                match read.find(&self.document) {
                    Ok(Some(target)) => {
                        target.write().add_referrer(item.to_weak());
                        drop(read);
                        item.write().set_target_reference(target.to_weak());
                        false
                    }
                    Ok(None) => true,
                    Err(err) => {
                        diagnostics.write().push(err);
                        true
                    }
                }
            })
            .collect::<Vec<WeakSymbol>>();
        self.diagnostics.extend(diagnostics.into_inner());
        self
    }

    #[cfg(not(feature = "rayon"))]
    pub fn resolve_checks(&mut self) -> &mut Self {
        self.unsolved_checks.retain(|item| {
            let item = match item.to_dyn() {
                Some(read) => read,
                None => return false,
            };
            let read = item.read();
            match read.check(&self.document, self.diagnostics) {
                Ok(()) => false,
                Err(()) => true,
            }
        });
        self
    }

    #[cfg(feature = "rayon")]
    pub fn resolve_checks(&mut self) -> &mut Self {
        use parking_lot::RwLock;
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

        let diagnostics = RwLock::new(vec![]);
        *self.unsolved_checks = self
            .unsolved_checks
            .par_iter()
            .cloned()
            .filter(|item| {
                let item = match item.to_dyn() {
                    Some(read) => read,
                    None => return false,
                };
                let read = item.read();
                match read.check(&self.document, &mut diagnostics.write()) {
                    Ok(()) => false,
                    Err(()) => true,
                }
            })
            .collect::<Vec<WeakSymbol>>();
        self.diagnostics.extend(diagnostics.into_inner());
        self
    }

    pub fn swap_ast(
        &'a mut self,
        root: &mut DynSymbol,
        edit_ranges: &Vec<(InputEdit, bool)>,
        ast_parser: &fn(
            &mut MainBuilder,
            Option<std::ops::Range<usize>>,
        ) -> Result<DynSymbol, lsp_types::Diagnostic>,
    ) -> &'a mut MainBuilder<'a> {
        for (edit, is_ws) in edit_ranges.iter() {
            let start_byte = edit.start_byte;
            let old_end_byte = edit.old_end_byte;
            let new_end_byte = edit.new_end_byte;

            let is_noop = old_end_byte == start_byte && new_end_byte == start_byte;
            if is_noop {
                continue;
            }

            root.edit_range(start_byte, (new_end_byte - old_end_byte) as isize);

            let node = self
                .document
                .cst
                .root_node()
                .descendant_for_byte_range(edit.start_byte, edit.new_end_byte);

            if let Some(node) = node {
                if let Some(node) = node.parent() {
                    if node.is_error() {
                        log::warn!("");
                        log::warn!("Node has an invalid syntax, aborting incremental update");
                        continue;
                    }
                }
                if node.is_extra() {
                    log::info!("");
                    log::info!("Node is extra, only update ranges");
                    continue;
                }
            }

            if *is_ws {
                log::info!("");
                log::info!("Whitespace edit, only update ranges");
                continue;
            }

            let parent_check = match root.read().must_check() {
                true => Some(root.to_weak()),
                false => None,
            };

            let result = root.write().dyn_update(
                start_byte,
                (new_end_byte - old_end_byte) as isize,
                parent_check,
                self,
            );
            match result {
                ControlFlow::Break(Err(e)) => {
                    self.diagnostics.push(e);
                }
                ControlFlow::Continue(()) => {
                    log::info!("");
                    log::info!("No incremental update available, root node will be reparsed");
                    log::info!("");
                    let mut ast_builder = ast_parser(self, None);
                    match ast_builder {
                        Ok(ref mut new_root) => {
                            root.swap(new_root);
                        }
                        Err(e) => {
                            self.diagnostics.push(e);
                        }
                    }
                }
                ControlFlow::Break(Ok(_)) => {}
            };
        }
        self
    }
}
