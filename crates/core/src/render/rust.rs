//! Rust renderer.
//!
//! Native port of `crates/cli/plugins/rust.lua`. The two implementations must
//! stay in sync byte for byte — `crates/cli/tests/render_parity.rs` enforces
//! that, so any change here needs the matching change in the plugin (and vice
//! versa).

use super::{deduplicate, sanitize_type_name};
use crate::schema::{Document, Field, NamedType, TypeExpr};
use std::collections::HashMap;

/// Reserved words that cannot appear bare as an identifier.
///
/// Listed in the same order as `RUST_KEYWORDS` in the plugin so the two are
/// easy to diff. Includes the reserved-for-future-use set (`become`, `priv`,
/// `typeof`, ...) because those are rejected by the compiler too.
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Renders `document` as `serde`-derived struct definitions.
///
/// Emits the `use serde::{Deserialize, Serialize};` header, then one `pub
/// struct` per named type, then — only when the root is not itself a named
/// type — a trailing `pub type` alias for the root.
pub fn render(document: &Document) -> String {
    let type_names = allocate_type_names(document);
    let mut out = String::from("use serde::{Deserialize, Serialize};\n\n");

    for named in &document.types {
        render_named_type(named, &type_names, &mut out);
    }

    if !matches!(document.root, TypeExpr::Named { .. }) {
        out.push_str("pub type ");
        out.push_str(&render_named_type_name(&document.root_name, &type_names));
        out.push_str(" = ");
        out.push_str(&render_type(&document.root, &type_names));
        out.push_str(";\n");
    }

    out
}

fn render_named_type(named: &NamedType, type_names: &TypeNames<'_>, out: &mut String) {
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str("pub struct ");
    out.push_str(&render_named_type_name(&named.name, type_names));
    out.push_str(" {\n");

    for (field, field_name) in named.fields.iter().zip(render_field_names(&named.fields)) {
        render_field(field, &field_name, type_names, out);
    }

    out.push_str("}\n\n");
}

fn render_field(field: &Field, field_name: &str, type_names: &TypeNames<'_>, out: &mut String) {
    // The rename is emitted whenever the generated identifier differs from the
    // raw JSON key *at all* — including when the only difference is the `r#`
    // raw-identifier prefix, so a key named `type` still gets both
    // `#[serde(rename = "type")]` and `pub r#type`.
    if field_name != field.name {
        out.push_str("    #[serde(rename = ");
        out.push_str(&rust_debug_escape(&field.name));
        out.push_str(")]\n");
    }

    let mut ty = render_type(&field.ty, type_names);
    if field.optional {
        ty = format!("Option<{ty}>");
    }

    out.push_str("    pub ");
    out.push_str(field_name);
    out.push_str(": ");
    out.push_str(&ty);
    out.push_str(",\n");
}

fn render_type(ty: &TypeExpr, type_names: &TypeNames<'_>) -> String {
    match ty {
        TypeExpr::Any => "serde_json::Value".to_string(),
        TypeExpr::Bool => "bool".to_string(),
        TypeExpr::Integer => "i64".to_string(),
        TypeExpr::UnsignedInteger => "u64".to_string(),
        TypeExpr::Float => "f64".to_string(),
        TypeExpr::String => "String".to_string(),
        TypeExpr::Named { name } => render_named_type_name(name, type_names),
        TypeExpr::Array { item } => format!("Vec<{}>", render_type(item, type_names)),
    }
}

/// Maps a type's *raw* (unsanitized) schema name to its allocated identifier.
type TypeNames<'a> = HashMap<&'a str, String>;

/// Allocates one unique identifier per declared type, keyed by raw name.
///
/// Keying by the raw name matters: two distinct schema names can sanitize to
/// the same base (`a-b` and `a.b` both become `a_b`), and the dedupe counter is
/// what keeps them apart. Named types are allocated first in declaration order,
/// then the root name — but only when the root is not itself a named type,
/// since in that case the root already has an entry.
fn allocate_type_names(document: &Document) -> TypeNames<'_> {
    let mut raw_names: Vec<&str> = document
        .types
        .iter()
        .map(|named| named.name.as_str())
        .collect();

    if !matches!(document.root, TypeExpr::Named { .. }) {
        raw_names.push(document.root_name.as_str());
    }

    let bases: Vec<String> = raw_names
        .iter()
        .map(|raw_name| render_type_name(raw_name))
        .collect();

    // Type names dedupe with no separator (`Foo`, `Foo2`), unlike field names.
    raw_names.into_iter().zip(deduplicate(&bases, "")).collect()
}

/// Resolves a type reference, falling back to a fresh sanitize when the raw
/// name was never allocated (an unreachable dangling reference in the schema).
fn render_named_type_name(name: &str, type_names: &TypeNames<'_>) -> String {
    type_names
        .get(name)
        .cloned()
        .unwrap_or_else(|| render_type_name(name))
}

/// Sanitizes a schema name into a type identifier.
///
/// `$` is not an identifier character in Rust, hence `allow_dollar = false`.
/// `Self` is syntactically valid but reserved as a type name, so it is pushed
/// aside to `SelfType`; this happens *after* sanitizing, so `Self!` (which
/// sanitizes to `Self_`) is left alone.
fn render_type_name(name: &str) -> String {
    let sanitized = sanitize_type_name(name, false);

    if sanitized == "Self" {
        format!("{sanitized}Type")
    } else {
        sanitized
    }
}

/// Generates the struct field identifiers for `fields`, in order.
fn render_field_names(fields: &[Field]) -> Vec<String> {
    let bases: Vec<String> = fields
        .iter()
        .map(|field| render_field_name(&field.name))
        .collect();

    // Field names dedupe with a `_` separator (`id`, `id_2`), unlike type
    // names. The keyword escape is applied last, so `type` and `type!` become
    // `r#type` and `type_2` — the suffixed form is no longer a keyword.
    deduplicate(&bases, "_")
        .iter()
        .map(|name| escape_rust_keyword(name))
        .collect()
}

/// Lowercases a JSON key into a snake-ish Rust field identifier.
///
/// This is deliberately *not* camelCase-to-snake_case splitting: it only
/// lowercases ASCII alphanumerics and collapses every run of other bytes into a
/// single `_`. So `userID` becomes `userid`, not `user_id`, and `display name`
/// becomes `display_name`.
///
/// Byte-wise and ASCII-only, because Lua's `%w` class matches ASCII only under
/// the C locale — every byte of a multi-byte UTF-8 sequence counts as a
/// separator individually. A leading digit gets an `_` prefix, trailing `_`
/// characters are stripped, and an empty result becomes `field`.
fn render_field_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut needs_separator = false;

    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() {
            if out.is_empty() && byte.is_ascii_digit() {
                out.push('_');
            }
            out.push(byte.to_ascii_lowercase() as char);
            needs_separator = false;
        } else if !out.is_empty() {
            needs_separator = true;
        }

        // Runs at the end of *every* iteration, matching the plugin's control
        // flow. Iterations that just pushed an alphanumeric cleared the flag,
        // so in practice only separator runs reach the push.
        if needs_separator && !out.ends_with('_') {
            out.push('_');
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "field".to_string()
    } else {
        out
    }
}

/// Prefixes reserved words with `r#` so they can be used as identifiers.
fn escape_rust_keyword(name: &str) -> String {
    if RUST_KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

/// Escapes `value` as a Rust string literal, including the surrounding quotes.
///
/// Not the same as JSON escaping: there is no `\b` or `\f` (Rust has no such
/// escapes), DEL is escaped, and non-printables use `\u{..}` with lowercase hex
/// and no zero padding rather than `\uXXXX`.
///
/// Iterates `chars`, not bytes, unlike the identifier helpers above: every
/// escaped codepoint here is ASCII, and non-ASCII characters must pass through
/// as their original UTF-8 bytes to match the plugin.
fn rust_debug_escape(value: &str) -> String {
    let mut out = String::from("\"");

    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            ch if (ch as u32) < 0x20 || ch as u32 == 0x7f => {
                let code = ch as u32;
                out.push_str(&format!("\\u{{{code:x}}}"));
            }
            ch => out.push(ch),
        }
    }

    out.push('"');
    out
}
