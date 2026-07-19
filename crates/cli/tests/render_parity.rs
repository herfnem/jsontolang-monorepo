//! Pins the two renderer implementations together.
//!
//! ponytail: ts/rust/go rendering logic is deliberately duplicated — the CLI
//! renders through `plugins/*.lua` (mlua), the browser playground renders
//! through `jsontolang_core::render` (native Rust), because mlua vendors C Lua
//! and does not target `wasm32-unknown-unknown`. See
//! `docs/jsontolang-monorepo-plan.md` for why and the upgrade path.
//!
//! This test is what makes that duplication safe: every case below is rendered
//! by *both* implementations and asserted byte-identical. If you touch a `.lua`
//! plugin or its `crates/core/src/render/*.rs` twin, this catches the drift.
//!
//! The plugins are loaded straight from disk via `include_str!` rather than
//! through `plugins::lookup_by_key`, so a developer who happens to have
//! `~/.config/jsontolang/plugins/` populated can't shadow them and turn this
//! into a false failure.

use jsontolang::plugins::LanguagePlugin;
use jsontolang::plugins::lua_plugin::LuaPlugin;
use jsontolang::schema::infer_document;
use serde_json::{Value, json};

const TYPESCRIPT_LUA: &str = include_str!("../plugins/typescript.lua");
const RUST_LUA: &str = include_str!("../plugins/rust.lua");
const GO_LUA: &str = include_str!("../plugins/go.lua");

/// `(root_name, json)` pairs exercised against every language.
///
/// Weighted toward the inputs where a hand port is most likely to drift from
/// the Lua original: non-ASCII keys (Lua's character classes are byte-wise
/// ASCII), control characters, quoting metacharacters, name collisions,
/// reserved words, and leading digits.
fn corpus() -> Vec<(&'static str, Value)> {
    vec![
        ("Root", json!({})),
        ("Root", json!({"name": "Neko"})),
        ("Root", json!(null)),
        ("Root", json!(42)),
        ("Root", json!("bare string")),
        ("Root", json!([])),
        ("Root", json!([1, 2, 3])),
        ("Root", json!([[1], [2, 3]])),
        ("Root", json!([1, "two", true])),
        ("Root", json!({"user": {"name": "Neko"}})),
        (
            "Root",
            json!({"items": [{"id": 1, "name": "A"}, {"id": 2}]}),
        ),
        ("Root", json!({"a": {"b": {"c": {"d": {"e": "deep"}}}}})),
        // Numeric widths: i64, u64-only, float, and the i64/u64 mix that
        // degrades to `any`.
        (
            "Root",
            json!({
                "signed": -1,
                "unsigned": 18446744073709551615u64,
                "float": 1.5,
                "mixed": [-1, 18446744073709551615u64],
            }),
        ),
        // Non-identifier keys.
        (
            "Root",
            json!({"display-name": "Neko", "123flag": true, "two words": "ok", "": 1}),
        ),
        // Keys that collide after sanitization.
        (
            "Root",
            json!({"display name": "A", "display-name": "B", "display.name": "C"}),
        ),
        // Rust keywords, raw-identifier escaping, and a keyword-plus-suffix.
        (
            "Root",
            json!({"type": 1, "type!": 2, "self": 3, "Self": 4, "r#type": 5, "try": 6}),
        ),
        // Go struct-tag metacharacters: backtick, backslash, double quote.
        (
            "Root",
            json!({"tick`key": 1, "path\\name": 2, "quote\"name": 3}),
        ),
        // Go custom-JSON triggers: the literal "-" key, a comma, control chars.
        (
            "Root",
            json!({"-": 1, "a,b": 2, "line\nfeed": 3, "tab\tkey": 4, "ok": 5}),
        ),
        // Every remaining JSON escape plus DEL, which rust.lua escapes and the
        // JSON escaper does not.
        (
            "Root",
            json!({"\u{8}\u{c}\r": 1, "del\u{7f}key": 2, "\u{1}\u{1f}": 3}),
        ),
        // Non-ASCII keys and type names — Lua classes these byte-wise, so a
        // char-wise port drifts here.
        ("Root", json!({"café": 1, "日本語": 2, "naïve-key": 3})),
        ("Root", json!({"emoji🎉key": 1})),
        // Non-ASCII, digit-leading, and empty root names.
        ("café root", json!({"child": {"name": "Neko"}})),
        ("123 root-name", json!({"child": {"name": "Neko"}})),
        ("", json!({"child": {"name": "Neko"}})),
        ("!!!", json!([{"id": 1}])),
        ("self", json!({"type": {"name": "Neko"}})),
        ("Self", json!({"child": {"name": "Neko"}})),
        // Root arrays, which emit a trailing type alias in all 3 languages.
        ("Root", json!([{"id": 1}, {"id": 2, "name": "Neko"}])),
        ("123 root-name", json!([{"id": 1}, {"id": 2}])),
        // Optionality: present-in-some, plus a null-only field.
        (
            "Root",
            json!({"items": [{"id": 1, "extra": null, "tags": ["a"]}, {"id": 2}]}),
        ),
    ]
}

#[test]
fn native_typescript_renderer_matches_the_lua_plugin() {
    assert_parity("typescript", TYPESCRIPT_LUA);
}

#[test]
fn native_rust_renderer_matches_the_lua_plugin() {
    assert_parity("rust", RUST_LUA);
}

#[test]
fn native_go_renderer_matches_the_lua_plugin() {
    assert_parity("go", GO_LUA);
}

fn assert_parity(lang: &str, lua_source: &str) {
    let plugin = LuaPlugin::load(lua_source, &format!("embedded:{lang}.lua"))
        .unwrap_or_else(|error| panic!("failed to load {lang}.lua: {error:#}"));

    for (root_name, value) in corpus() {
        let document = infer_document(root_name, &value)
            .unwrap_or_else(|error| panic!("infer_document failed for {value}: {error:#}"));

        let via_lua = plugin
            .render(&document)
            .unwrap_or_else(|error| panic!("{lang}.lua render failed for {value}: {error:#}"));
        let via_native = jsontolang_core::render(lang, &document)
            .unwrap_or_else(|error| panic!("native {lang} render failed for {value}: {error:#}"));

        assert_eq!(
            via_lua, via_native,
            "\n{lang} renderers disagree\n  root: {root_name:?}\n  json: {value}\n\
             --- via {lang}.lua ---\n{via_lua}\
             --- via crates/core/src/render/{lang}.rs ---\n{via_native}"
        );
    }
}
