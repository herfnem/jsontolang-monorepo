//! Native Rust renderers for the built-in target languages.
//!
//! ponytail: rendering logic is duplicated between here and
//! `crates/cli/plugins/*.lua` — the CLI renders through sandboxed Lua plugins,
//! this module renders natively so the browser playground can run without
//! mlua (which vendors C Lua and does not target `wasm32-unknown-unknown`).
//! See `docs/jsontolang-monorepo-plan.md` for why and the upgrade path.
//! `crates/cli/tests/render_parity.rs` pins the two implementations together.

pub mod go;
pub mod rust;
pub mod typescript;

use crate::schema::Document;
use anyhow::{Result, bail};

/// Languages `render` accepts, in documentation order.
pub const BUILTIN_LANGS: &[&str] = &["typescript", "rust", "go"];

/// Renders `document` as `lang` source text.
///
/// This is the wasm/playground entry point. The CLI deliberately does *not*
/// call it — it goes through its Lua plugins instead, so CLI output stays
/// byte-identical to the pre-monorepo `jsontolang` binary.
pub fn render(lang: &str, document: &Document) -> Result<String> {
    match lang {
        "typescript" => Ok(typescript::render(document)),
        "rust" => Ok(rust::render(document)),
        "go" => Ok(go::render(document)),
        other => bail!(
            "unknown language `{other}`, available: {}",
            BUILTIN_LANGS.join(", ")
        ),
    }
}

/// Escapes `value` as a JSON string literal, including the surrounding quotes.
///
/// Mirrors `json_escape` in both `typescript.lua` and `go.lua`, which are
/// byte-identical to each other.
///
/// Builds bytes rather than chars on purpose: the Lua original concatenates raw
/// bytes, so a byte `>= 0x80` must be passed through untouched. Pushing it as a
/// `char` instead would re-encode it as a two-byte UTF-8 sequence and corrupt
/// every non-ASCII key.
pub(crate) fn json_escape(value: &str) -> String {
    let mut out: Vec<u8> = vec![b'"'];

    for byte in value.bytes() {
        match byte {
            b'"' => out.extend_from_slice(b"\\\""),
            b'\\' => out.extend_from_slice(b"\\\\"),
            0x08 => out.extend_from_slice(b"\\b"),
            0x09 => out.extend_from_slice(b"\\t"),
            0x0a => out.extend_from_slice(b"\\n"),
            0x0c => out.extend_from_slice(b"\\f"),
            0x0d => out.extend_from_slice(b"\\r"),
            b if b < 0x20 => out.extend_from_slice(format!("\\u{b:04x}").as_bytes()),
            b => out.push(b),
        }
    }

    out.push(b'"');

    // Only ASCII escapes were substituted, and they only ever replaced ASCII
    // bytes, so the remaining multi-byte sequences are still intact.
    String::from_utf8(out)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

/// Turns an inferred type name into a valid identifier for a C-like language.
///
/// Mirrors `render_type_name`/`sanitize_type_name` in the Lua plugins, which
/// differ only in whether `$` counts as an identifier character (TypeScript:
/// yes; Rust and Go: no).
///
/// Iterates *bytes*, not chars, because Lua's `%a`/`%w` classes are ASCII-only
/// under the C locale — so every byte of a multi-byte UTF-8 sequence is treated
/// as invalid individually. Matching that byte-wise behaviour is what keeps
/// this in parity with the Lua plugins on non-ASCII input.
pub(crate) fn sanitize_type_name(name: &str, allow_dollar: bool) -> String {
    let is_start = |b: u8| b == b'_' || (allow_dollar && b == b'$') || b.is_ascii_alphabetic();
    let is_char = |b: u8| b == b'_' || (allow_dollar && b == b'$') || b.is_ascii_alphanumeric();

    // Every byte pushed below has already been classified as ASCII, so building
    // a `String` directly (rather than a byte buffer) is sound.
    let mut out = String::with_capacity(name.len());

    for (index, byte) in name.bytes().enumerate() {
        let valid = if index == 0 {
            is_start(byte)
        } else {
            is_char(byte)
        };

        if valid {
            out.push(byte as char);
        } else if index == 0 {
            out.push('_');
            if byte.is_ascii_digit() {
                out.push(byte as char);
            }
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }

    if out.is_empty() {
        "Root".to_string()
    } else {
        out
    }
}

/// Assigns a unique name per occurrence, appending `2`, `3`, ... to repeats.
///
/// `separator` is `""` for type names and `"_"` for Rust field names, matching
/// `allocate_type_names` and `render_field_names` in the Lua plugins.
pub(crate) fn deduplicate(bases: &[String], separator: &str) -> Vec<String> {
    let mut used: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut names = Vec::with_capacity(bases.len());

    for base in bases {
        let count = used.entry(base.as_str()).or_insert(0);
        *count += 1;

        names.push(if *count == 1 {
            base.clone()
        } else {
            format!("{base}{separator}{count}")
        });
    }

    names
}
