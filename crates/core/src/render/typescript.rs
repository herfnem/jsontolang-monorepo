//! TypeScript renderer.
//!
//! Native port of `crates/cli/plugins/typescript.lua`. The two implementations
//! must stay in sync byte for byte — `crates/cli/tests/render_parity.rs`
//! enforces that, so any change here needs the matching change in the plugin
//! (and vice versa).

use super::{json_escape, sanitize_type_name};
use crate::schema::{Document, Field, NamedType, TypeExpr};

/// Renders `document` as TypeScript interface and type-alias declarations.
///
/// Emits one `export interface` per named type, then — only when the root is
/// not itself a named type — a trailing `export type` alias for the root.
pub fn render(document: &Document) -> String {
    let mut out = String::new();

    for named in &document.types {
        render_named_type(named, &mut out);
    }

    if !matches!(document.root, TypeExpr::Named { .. }) {
        out.push_str("export type ");
        out.push_str(&render_type_name(&document.root_name));
        out.push_str(" = ");
        out.push_str(&render_type(&document.root));
        out.push_str(";\n");
    }

    out
}

fn render_named_type(named: &NamedType, out: &mut String) {
    out.push_str("export interface ");
    out.push_str(&render_type_name(&named.name));
    out.push_str(" {\n");

    for field in &named.fields {
        render_field(field, out);
    }

    out.push_str("}\n\n");
}

fn render_field(field: &Field, out: &mut String) {
    out.push_str("  ");
    out.push_str(&render_property_name(&field.name));
    if field.optional {
        out.push('?');
    }
    out.push_str(": ");
    out.push_str(&render_type(&field.ty));
    out.push_str(";\n");
}

fn render_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Any => "any".to_string(),
        TypeExpr::Bool => "boolean".to_string(),
        TypeExpr::Integer | TypeExpr::UnsignedInteger | TypeExpr::Float => "number".to_string(),
        TypeExpr::String => "string".to_string(),
        TypeExpr::Named { name } => render_type_name(name),
        TypeExpr::Array { item } => format!("{}[]", render_type(item)),
    }
}

/// `$` is an identifier character in TypeScript, unlike in Rust or Go.
fn render_type_name(name: &str) -> String {
    sanitize_type_name(name, true)
}

/// Emits a property name bare when it is a valid identifier, quoted otherwise.
fn render_property_name(name: &str) -> String {
    if is_typescript_identifier(name) {
        name.to_string()
    } else {
        json_escape(name)
    }
}

/// Reports whether `name` can appear unquoted as an object property.
///
/// Deliberately byte-wise and ASCII-only: Lua's `%a`/`%w` classes match ASCII
/// only under the C locale, so the plugin rejects any name containing a
/// multi-byte UTF-8 sequence. Iterating `chars()` here would accept names the
/// plugin quotes, breaking parity.
fn is_typescript_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();

    let Some(first) = bytes.next() else {
        return false;
    };

    if !(first == b'_' || first == b'$' || first.is_ascii_alphabetic()) {
        return false;
    }

    bytes.all(|byte| byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric())
}
