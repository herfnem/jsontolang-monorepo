//! Go renderer.
//!
//! Native port of `crates/cli/plugins/go.lua`. The two implementations must
//! stay in sync byte for byte — `crates/cli/tests/render_parity.rs` enforces
//! that, so any change here needs the matching change in the plugin (and vice
//! versa).

use super::{deduplicate, json_escape, sanitize_type_name};
use crate::schema::{Document, Field, NamedType, TypeExpr};

/// Renders `document` as a Go `package models` source file.
///
/// Emits one `struct` per named type, plus `UnmarshalJSON`/`MarshalJSON`
/// methods for the structs holding at least one JSON key a struct tag cannot
/// express, then — only when the root is not itself a named type — a trailing
/// type alias for the root.
pub fn render(document: &Document) -> String {
    let mut out = String::from("package models\n\n");

    if document.types.iter().any(named_type_needs_custom_json) {
        out.push_str("import \"encoding/json\"\n\n");
    }

    for named in &document.types {
        render_named_type(named, &mut out);
    }

    if !matches!(document.root, TypeExpr::Named { .. }) {
        out.push_str("type ");
        out.push_str(&render_type_name(&document.root_name));
        out.push_str(" = ");
        out.push_str(&render_type(&document.root));
        out.push('\n');
    }

    out
}

fn render_named_type(named: &NamedType, out: &mut String) {
    let field_names = render_field_names(&named.fields);

    out.push_str("type ");
    out.push_str(&render_type_name(&named.name));
    out.push_str(" struct {\n");

    for (field, field_name) in named.fields.iter().zip(&field_names) {
        out.push('\t');
        out.push_str(field_name);
        out.push(' ');
        out.push_str(&render_field_type(field));
        out.push(' ');
        out.push_str(&render_struct_tag(&field.name));
        out.push('\n');
    }

    out.push_str("}\n\n");

    if named_type_needs_custom_json(named) {
        render_custom_json_methods(named, &field_names, out);
    }
}

/// Emits hand-rolled marshal/unmarshal methods for the keys `encoding/json`
/// cannot round-trip through a struct tag.
///
/// Only the offending fields get explicit handling; everything else still goes
/// through the generated `plain` alias so the ordinary struct tags keep working.
fn render_custom_json_methods(named: &NamedType, field_names: &[String], out: &mut String) {
    let type_name = render_type_name(&named.name);

    out.push_str("func (value *");
    out.push_str(&type_name);
    out.push_str(") UnmarshalJSON(data []byte) error {\n");
    out.push_str("\ttype plain ");
    out.push_str(&type_name);
    out.push('\n');
    out.push_str("\traw := map[string]json.RawMessage{}\n");
    out.push_str("\tif err := json.Unmarshal(data, &raw); err != nil {\n");
    out.push_str("\t\treturn err\n");
    out.push_str("\t}\n");

    for (field, field_name) in named.fields.iter().zip(field_names) {
        if !needs_custom_json_key(&field.name) {
            continue;
        }

        let key = json_escape(&field.name);

        out.push_str("\tif field, ok := raw[");
        out.push_str(&key);
        out.push_str("]; ok {\n");
        out.push_str("\t\tif err := json.Unmarshal(field, &value.");
        out.push_str(field_name);
        out.push_str("); err != nil {\n");
        out.push_str("\t\t\treturn err\n");
        out.push_str("\t\t}\n");
        out.push_str("\t}\n");
        out.push_str("\tdelete(raw, ");
        out.push_str(&key);
        out.push_str(")\n");
    }

    out.push_str("\trest, err := json.Marshal(raw)\n");
    out.push_str("\tif err != nil {\n");
    out.push_str("\t\treturn err\n");
    out.push_str("\t}\n");
    out.push_str("\treturn json.Unmarshal(rest, (*plain)(value))\n");
    out.push_str("}\n\n");

    out.push_str("func (value ");
    out.push_str(&type_name);
    out.push_str(") MarshalJSON() ([]byte, error) {\n");
    out.push_str("\ttype plain ");
    out.push_str(&type_name);
    out.push('\n');
    out.push_str("\tencoded, err := json.Marshal(plain(value))\n");
    out.push_str("\tif err != nil {\n");
    out.push_str("\t\treturn nil, err\n");
    out.push_str("\t}\n");
    out.push_str("\traw := map[string]json.RawMessage{}\n");
    out.push_str("\tif err := json.Unmarshal(encoded, &raw); err != nil {\n");
    out.push_str("\t\treturn nil, err\n");
    out.push_str("\t}\n");
    out.push_str("\tvar payload []byte\n");

    for (field, field_name) in named.fields.iter().zip(field_names) {
        if !needs_custom_json_key(&field.name) {
            continue;
        }

        out.push_str("\tpayload, err = json.Marshal(value.");
        out.push_str(field_name);
        out.push_str(")\n");
        out.push_str("\tif err != nil {\n");
        out.push_str("\t\treturn nil, err\n");
        out.push_str("\t}\n");
        out.push_str("\traw[");
        out.push_str(&json_escape(&field.name));
        out.push_str("] = payload\n");
    }

    out.push_str("\treturn json.Marshal(raw)\n");
    out.push_str("}\n\n");
}

fn named_type_needs_custom_json(named: &NamedType) -> bool {
    named
        .fields
        .iter()
        .any(|field| needs_custom_json_key(&field.name))
}

fn render_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Any => "interface{}".to_string(),
        TypeExpr::Bool => "bool".to_string(),
        TypeExpr::Integer => "int64".to_string(),
        TypeExpr::UnsignedInteger => "uint64".to_string(),
        TypeExpr::Float => "float64".to_string(),
        TypeExpr::String => "string".to_string(),
        TypeExpr::Named { name } => render_type_name(name),
        TypeExpr::Array { item } => format!("[]{}", render_type(item)),
    }
}

/// Optional fields become pointers so a missing key stays distinguishable from
/// the zero value.
fn render_field_type(field: &Field) -> String {
    let ty = render_type(&field.ty);

    if field.optional {
        return format!("*{ty}");
    }

    ty
}

/// `$` is not an identifier character in Go, unlike in TypeScript.
fn render_type_name(name: &str) -> String {
    sanitize_type_name(name, false)
}

/// Assigns every field an exported Go name, disambiguating collisions.
///
/// Repeats get a bare counter suffix (`Name`, `Name2`, `Name3`) — no
/// separator — because Go field names are camel case, not snake case.
fn render_field_names(fields: &[Field]) -> Vec<String> {
    let bases: Vec<String> = fields
        .iter()
        .map(|field| render_field_name(&field.name))
        .collect();

    deduplicate(&bases, "")
}

/// Turns a JSON key into an exported Go field name.
///
/// Splits on every non-alphanumeric byte, then upper-cases the first byte of
/// each run and lower-cases the rest — so `userID` becomes `Userid`, not
/// `UserID`. A run of digits at the very start is prefixed with `X` to keep the
/// identifier legal, and a name with no alphanumerics at all falls back to
/// `Field`.
///
/// Deliberately byte-wise and ASCII-only: Lua's `%w` class and `string.upper`
/// only cover ASCII under the C locale, so every byte of a multi-byte UTF-8
/// sequence acts as a separator. Iterating `chars()` here would keep letters the
/// plugin drops, breaking parity.
fn render_field_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 1);
    let mut part: Vec<u8> = Vec::new();

    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() {
            part.push(byte);
        } else {
            flush_field_name_part(&mut part, &mut out);
        }
    }

    flush_field_name_part(&mut part, &mut out);

    if out.is_empty() {
        return "Field".to_string();
    }

    out
}

/// Appends the pending run of alphanumerics to `out`, then clears it.
fn flush_field_name_part(part: &mut Vec<u8>, out: &mut String) {
    let Some(&first) = part.first() else {
        return;
    };

    // Only the leading run can start the identifier, so only it needs the
    // digit guard.
    if out.is_empty() && first.is_ascii_digit() {
        out.push('X');
    }

    // Every byte in `part` is ASCII, so the `as char` casts are lossless.
    out.push(first.to_ascii_uppercase() as char);
    for byte in &part[1..] {
        out.push(byte.to_ascii_lowercase() as char);
    }

    part.clear();
}

/// Reports whether `name` needs the generated `UnmarshalJSON`/`MarshalJSON`
/// methods rather than a plain struct tag.
///
/// `encoding/json` reads `-` as "skip this field" and treats a comma as the
/// start of tag options, so neither can appear in a tag as a literal key.
/// Control characters are excluded too, since they cannot survive a Go raw
/// string literal. Distinct from [`needs_interpreted_tag_literal`], which only
/// decides how to *quote* a key the tag can still carry.
fn needs_custom_json_key(name: &str) -> bool {
    name == "-" || name.contains(',') || has_control_char(name)
}

/// Reports whether `name`'s struct tag has to be an interpreted (double-quoted)
/// Go string literal instead of a raw (backtick) one.
///
/// A raw literal cannot contain a backtick, and the tag text itself already
/// contains the JSON-escaped key, so a backslash or double quote in `name`
/// would also have to be spelled out. Distinct from [`needs_custom_json_key`]:
/// these keys still round-trip through the tag, they just need heavier quoting.
fn needs_interpreted_tag_literal(name: &str) -> bool {
    name.bytes().any(|byte| matches!(byte, b'`' | b'\\' | b'"')) || has_control_char(name)
}

/// Renders the `json:"..."` struct tag for `name`, quoting as lightly as it can.
///
/// Falls back to `json:"-"` for keys [`needs_custom_json_key`] rejects; those
/// are carried by the generated marshal methods instead.
fn render_struct_tag(name: &str) -> String {
    if needs_custom_json_key(name) {
        return "`json:\"-\"`".to_string();
    }

    render_json_tag(name)
}

fn render_json_tag(name: &str) -> String {
    let tag = format!("json:{}", json_escape(name));

    if needs_interpreted_tag_literal(name) {
        return json_escape(&tag);
    }

    format!("`{tag}`")
}

fn has_control_char(value: &str) -> bool {
    value.bytes().any(is_control_byte)
}

// ponytail: control-char detection is ASCII 0x00-0x1F + DEL only, not full
// Unicode C1 range (0x80-0x9F) like Rust's char::is_control(); upgrade if a
// plugin ever needs to key on non-ASCII control characters.
fn is_control_byte(byte: u8) -> bool {
    byte < 0x20 || byte == 0x7f
}
