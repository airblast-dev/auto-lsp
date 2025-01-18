#![allow(rustdoc::private_intra_doc_links)]
//! # Auto LSP
//!
//! A Rust crate for creating [Abstract Syntax Trees](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (AST)
//! and [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) (LSP) servers.
//!
//! `auto_lsp` is designed to be as language-agnostic as possible, allowing any Tree-sitter grammar to be used.
//!
//! Defining a simple AST involves two steps: writing the queries and then defining the corresponding AST structures in Rust.
//!
//! ## Quick example
//!
//! Let's say you have a toy language with a root node named **document** containing a list of **function** nodes,
//! each containing a unique **name**.
//!
//! A simple query file to capture the root document and function names:
//!
//! ```lisp
//! (document) @document
//! (function
//!     (name) @name) @function
//! ```
//!
//! The corresponding AST definition in Rust:
//!
//! ```
//! # use auto_lsp::core::ast::*;
//! # use auto_lsp::macros::seq;
//!
//! #[seq(query_name = "document", kind(symbol()))]
//! struct Document {
//!    /// A collection of functions defined in the document
//!    functions: Vec<Function>
//! }
//!
//! #[seq(query_name = "function", kind(symbol()))]
//! struct Function {
//!    /// The name of the function
//!    name: Name
//! }
//!
//! #[seq(query_name = "name", kind(symbol()))]
//! struct Name {}  
//! ```
//!
//! Now that you have your AST defined, you can:
//!  - Implements the [LSP traits](core::ast) and create a LSP server (with the `lsp_server` feature).
//!  - Add your own logic for testing purposes, code_generation, etc.
//!
//! You can find more examples in the `tests` folder.
//!
//! ## Features
//! - `lsp_server`: Enable the LSP server.
//! - `wasm`: Enable wasm support.
//! - `rayon`: Enable rayon support (not compatible with `wasm`).
//! - `python_test`: Enable the python workspace mock for testing purposes.
//!

/// A mock Python workspace used for testing purposes.
/// This module is only available with the `python_test` feature enabled or during tests.
#[cfg(any(feature = "python_test", test))]
pub mod python_workspace;
/// LSP server (enabled with feature `lsp_server`)
#[cfg(feature = "lsp_server")]
pub mod server;
#[cfg(test)]
pub mod tests;

/// Core functionalities of the crate
pub mod core {
    // Not public API. Referenced by macro-generated code.
    #[doc(hidden)]
    pub mod build {
        pub use auto_lsp_core::build::*;
    }

    pub use auto_lsp_core::ast;
    pub use auto_lsp_core::semantic_tokens;
    pub use auto_lsp_core::workspace;
    pub use auto_lsp_core::{builder_error, builder_warning};
}
/// [`macros::seq`] and [`macros::choice`] macros
pub use auto_lsp_macros as macros;

#[doc(hidden)]
pub use constcat;
pub use lsp_types;
pub use parking_lot;
#[cfg(feature = "rayon")]
pub use rayon;
pub use texter;
pub use tree_sitter;
