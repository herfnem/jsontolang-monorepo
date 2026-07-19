//! Browser bindings for jsontolang.
//!
//! Thin pass-through over `jsontolang-core`: no plugin discovery, no
//! filesystem, no mlua. Custom `.lua` plugins stay CLI-only because mlua
//! vendors C Lua through a `cc` build script and does not target
//! `wasm32-unknown-unknown`, so the playground offers the built-in languages
//! rendered by `jsontolang_core::render` instead.

use jsontolang_core::{BUILTIN_LANGS, infer_document, render};
use wasm_bindgen::prelude::*;

/// Parses `json` and returns the inferred schema `Document` as a JS object.
#[wasm_bindgen]
pub fn infer(root: &str, json: &str) -> Result<JsValue, JsValue> {
    let value = parse(json)?;
    let document = infer_document(root, &value).map_err(to_js_error)?;

    serde_wasm_bindgen::to_value(&document).map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Parses `json` and renders its inferred schema as `lang` source text.
#[wasm_bindgen]
pub fn render_types(root: &str, json: &str, lang: &str) -> Result<String, JsValue> {
    let value = parse(json)?;
    let document = infer_document(root, &value).map_err(to_js_error)?;

    render(lang, &document).map_err(to_js_error)
}

/// Languages [`render_types`] accepts, for populating the language picker.
#[wasm_bindgen]
pub fn builtin_languages() -> Vec<String> {
    BUILTIN_LANGS.iter().map(|lang| lang.to_string()).collect()
}

fn parse(json: &str) -> Result<serde_json::Value, JsValue> {
    serde_json::from_str(json)
        .map_err(|error| JsValue::from_str(&format!("invalid JSON input: {error}")))
}

fn to_js_error(error: anyhow::Error) -> JsValue {
    JsValue::from_str(&error.to_string())
}
