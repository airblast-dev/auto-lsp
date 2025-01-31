use auto_lsp_core::semantic_tokens::SemanticTokensBuilder;

use lsp_types::{SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensResult};

use crate::server::session::{Session, WORKSPACES};

impl Session {
    /// Get all semantic tokens for a document.
    pub fn get_semantic_tokens_full(
        &mut self,
        params: SemanticTokensParams,
    ) -> anyhow::Result<SemanticTokensResult> {
        let uri = &params.text_document.uri;
        let workspace = WORKSPACES.lock();

        let workspace = workspace
            .get(&uri)
            .ok_or(anyhow::anyhow!("Workspace not found"))?;

        let mut builder = SemanticTokensBuilder::new(0.to_string());

        workspace.ast.iter().for_each(|p| {
            p.read()
                .build_semantic_tokens(&workspace.document, &mut builder)
        });

        Ok(SemanticTokensResult::Tokens(builder.build()))
    }

    /// Get semantic tokens for a range in a document.
    pub fn get_semantic_tokens_range(
        &mut self,
        params: SemanticTokensRangeParams,
    ) -> anyhow::Result<SemanticTokensResult> {
        let uri = &params.text_document.uri;
        let workspace = WORKSPACES.lock();

        let workspace = workspace
            .get(&uri)
            .ok_or(anyhow::anyhow!("Workspace not found"))?;

        let mut builder = SemanticTokensBuilder::new(0.to_string());

        workspace.ast.iter().for_each(|p| {
            p.read()
                .build_semantic_tokens(&workspace.document, &mut builder)
        });

        Ok(SemanticTokensResult::Tokens(builder.build()))
    }
}

#[macro_export]
macro_rules! define_semantic_token_types {
    (
        standard {
            $($ts_name: expr => $standard:ident),*$(,)?
        }

    ) => {
        $(pub const $standard: auto_lsp::lsp_types::SemanticTokenType = auto_lsp::lsp_types::SemanticTokenType::$standard;)*

        pub const SUPPORTED_TYPES: &[auto_lsp::lsp_types::SemanticTokenType] = &[
            $(auto_lsp::lsp_types::SemanticTokenType::$standard,)*
        ];

        pub static TOKEN_TYPES: phf::OrderedMap<&'static str, auto_lsp::lsp_types::SemanticTokenType> = phf::phf_ordered_map! {
            $( $ts_name => auto_lsp::lsp_types::SemanticTokenType::$standard,)*
        };
    };
}

#[macro_export]
macro_rules! define_semantic_token_modifiers {
    (
        standard {
            $($ts_name: expr => $standard:ident),*$(,)?
        }

    ) => {

        $(pub const $standard: auto_lsp::lsp_types::SemanticTokenModifier = auto_lsp::lsp_types::SemanticTokenModifier::$standard;)*

        pub const SUPPORTED_MODIFIERS: &[auto_lsp::lsp_types::SemanticTokenModifier] = &[
            $(auto_lsp::lsp_types::SemanticTokenModifier::$standard,)*
        ];

        pub static TOKEN_MODIFIERS: phf::OrderedMap<&'static str, auto_lsp::lsp_types::SemanticTokenModifier> = phf::phf_ordered_map! {
            $( $ts_name => auto_lsp::lsp_types::SemanticTokenModifier::$standard,)*
        };
    };
}
