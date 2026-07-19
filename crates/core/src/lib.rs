//! Pure, I/O-free core of jsontolang: JSON -> schema IR -> generated type
//! definitions.
//!
//! This crate has no `mlua` dependency and no filesystem access, so it builds
//! for `wasm32-unknown-unknown`. The CLI (`jsontolang`) layers Lua plugin
//! discovery on top of [`schema`]; the browser playground (`jsontolang-wasm`)
//! layers [`render`] on top of it instead.

pub mod render;
pub mod schema;

pub use render::{BUILTIN_LANGS, render};
pub use schema::{Document, Field, NamedType, TypeExpr, infer_document};
