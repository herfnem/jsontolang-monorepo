//! Pins the native renderers' output without reaching for the CLI crate.
//!
//! These assertions are ported from `crates/cli/tests/plugin_output.rs`, so
//! they read as a spec for what each language emits. Full byte-for-byte
//! equivalence with the Lua plugins is enforced separately, and more
//! aggressively, by `crates/cli/tests/render_parity.rs` — that test needs mlua
//! and therefore can't live in this crate.

use jsontolang_core::{infer_document, render};
use serde_json::json;

fn rendered(lang: &str, root: &str, value: serde_json::Value) -> String {
    let document = infer_document(root, &value).unwrap();
    render(lang, &document).unwrap()
}

#[test]
fn rejects_unknown_languages_and_lists_the_available_ones() {
    let document = infer_document("Root", &json!({})).unwrap();
    let message = render("cobol", &document).unwrap_err().to_string();

    assert!(message.contains("unknown language `cobol`"));
    assert!(message.contains("typescript"));
    assert!(message.contains("rust"));
    assert!(message.contains("go"));
}

#[test]
fn renders_typescript_nested_interfaces() {
    let output = rendered("typescript", "Root", json!({"user": {"name": "Neko"}}));

    assert_eq!(
        output,
        concat!(
            "export interface Root {\n",
            "  user: RootUser;\n",
            "}\n\n",
            "export interface RootUser {\n",
            "  name: string;\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_typescript_optional_properties() {
    let output = rendered(
        "typescript",
        "Root",
        json!({"items": [{"id": 1, "name": "A"}, {"id": 2}]}),
    );

    assert!(output.contains("export interface RootItemsItem {"));
    assert!(output.contains("name?: string;"));
}

#[test]
fn renders_typescript_non_identifier_property_keys() {
    let output = rendered(
        "typescript",
        "Root",
        json!({"display-name": "Neko", "123flag": true, "two words": "ok"}),
    );

    assert!(output.contains("\"display-name\": string;"));
    assert!(output.contains("\"123flag\": boolean;"));
    assert!(output.contains("\"two words\": string;"));
}

#[test]
fn sanitizes_invalid_root_names_for_typescript_declarations() {
    let output = rendered(
        "typescript",
        "123 root-name",
        json!({"child": {"name": "Neko"}}),
    );

    assert!(output.contains("export interface _123RootName {"));
    assert!(output.contains("child: _123RootNameChild;"));
    assert!(output.contains("export interface _123RootNameChild {"));
}

#[test]
fn renders_typescript_root_arrays_as_exported_type_aliases() {
    let output = rendered(
        "typescript",
        "123 root-name",
        json!([{"id": 1}, {"id": 2, "name": "Neko"}]),
    );

    assert!(output.contains("export interface _123RootNameItem {"));
    assert!(output.contains("name?: string;"));
    assert!(output.contains("export type _123RootName = _123RootNameItem[];"));
}

#[test]
fn renders_rust_structs_with_serde_renames() {
    let output = rendered(
        "rust",
        "Root",
        json!({
            "display-name": "Neko",
            "items": [{"id": 1}, {"id": 2, "extra": null}],
            "child": {"two words": true}
        }),
    );

    assert_eq!(
        output,
        concat!(
            "use serde::{Deserialize, Serialize};\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct Root {\n",
            "    pub child: RootChild,\n",
            "    #[serde(rename = \"display-name\")]\n",
            "    pub display_name: String,\n",
            "    pub items: Vec<RootItemsItem>,\n",
            "}\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct RootChild {\n",
            "    #[serde(rename = \"two words\")]\n",
            "    pub two_words: bool,\n",
            "}\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct RootItemsItem {\n",
            "    pub extra: Option<serde_json::Value>,\n",
            "    pub id: i64,\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_rust_root_arrays_as_type_aliases() {
    let output = rendered(
        "rust",
        "Root",
        json!([{"id": 1}, {"id": 2, "name": "Neko"}]),
    );

    assert!(output.contains("pub struct RootItem {"));
    assert!(output.contains("pub name: Option<String>,"));
    assert!(output.contains("pub type Root = Vec<RootItem>;"));
}

#[test]
fn renders_rust_colliding_and_keyword_field_names_deterministically() {
    let output = rendered(
        "rust",
        "Root",
        json!({"display name": "A", "display-name": "B", "type": 1, "type!": 2}),
    );

    assert_eq!(
        output,
        concat!(
            "use serde::{Deserialize, Serialize};\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct Root {\n",
            "    #[serde(rename = \"display name\")]\n",
            "    pub display_name: String,\n",
            "    #[serde(rename = \"display-name\")]\n",
            "    pub display_name_2: String,\n",
            "    #[serde(rename = \"type\")]\n",
            "    pub r#type: i64,\n",
            "    #[serde(rename = \"type!\")]\n",
            "    pub type_2: i64,\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_rust_u64_only_integer_values_as_u64() {
    let output = rendered(
        "rust",
        "Root",
        json!({
            "samples": [
                {"value": 9223372036854775808u64},
                {"value": 18446744073709551615u64}
            ]
        }),
    );

    assert!(output.contains("pub value: u64,"), "{output}");
}

#[test]
fn renders_rust_reserved_root_type_names_safely() {
    let output = rendered("rust", "self", json!({"child": {"name": "Neko"}}));

    assert_eq!(
        output,
        concat!(
            "use serde::{Deserialize, Serialize};\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct SelfType {\n",
            "    pub child: SelfChild,\n",
            "}\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct SelfChild {\n",
            "    pub name: String,\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_rust_reserved_and_inferred_type_name_collisions_deterministically() {
    let output = rendered("rust", "self", json!({"type": {"name": "Neko"}}));

    assert_eq!(
        output,
        concat!(
            "use serde::{Deserialize, Serialize};\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct SelfType {\n",
            "    #[serde(rename = \"type\")]\n",
            "    pub r#type: SelfType2,\n",
            "}\n\n",
            "#[derive(Debug, Clone, Serialize, Deserialize)]\n",
            "pub struct SelfType2 {\n",
            "    pub name: String,\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_go_structs_with_json_tags_and_optionality() {
    let output = rendered(
        "go",
        "Root",
        json!({
            "items": [
                {
                    "id": 1,
                    "name": "A",
                    "child": {"two words": true},
                    "aliases": ["A"],
                    "extra": null
                },
                {"id": 2}
            ]
        }),
    );

    assert_eq!(
        output,
        concat!(
            "package models\n\n",
            "type Root struct {\n",
            "\tItems []RootItemsItem `json:\"items\"`\n",
            "}\n\n",
            "type RootItemsItem struct {\n",
            "\tAliases *[]string `json:\"aliases\"`\n",
            "\tChild *RootItemsItemChild `json:\"child\"`\n",
            "\tExtra *interface{} `json:\"extra\"`\n",
            "\tId int64 `json:\"id\"`\n",
            "\tName *string `json:\"name\"`\n",
            "}\n\n",
            "type RootItemsItemChild struct {\n",
            "\tTwoWords bool `json:\"two words\"`\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_go_colliding_field_names_deterministically() {
    let output = rendered(
        "go",
        "Root",
        json!({"display name": "A", "display-name": "B"}),
    );

    assert_eq!(
        output,
        concat!(
            "package models\n\n",
            "type Root struct {\n",
            "\tDisplayName string `json:\"display name\"`\n",
            "\tDisplayName2 string `json:\"display-name\"`\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_go_json_tags_with_backticks_using_valid_string_literals() {
    let output = rendered("go", "Root", json!({"tick`key": 1}));

    assert_eq!(
        output,
        concat!(
            "package models\n\n",
            "type Root struct {\n",
            "\tTickKey int64 \"json:\\\"tick`key\\\"\"\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_go_json_tags_with_quotes_and_backslashes_using_valid_string_literals() {
    let output = rendered("go", "Root", json!({"path\\name": 1, "quote\"name": 2}));

    assert_eq!(
        output,
        concat!(
            "package models\n\n",
            "type Root struct {\n",
            "\tPathName int64 \"json:\\\"path\\\\\\\\name\\\"\"\n",
            "\tQuoteName int64 \"json:\\\"quote\\\\\\\"name\\\"\"\n",
            "}\n\n"
        )
    );
}

#[test]
fn renders_go_u64_only_integer_values_as_uint64() {
    let output = rendered(
        "go",
        "Root",
        json!({
            "samples": [
                {"value": 9223372036854775808u64},
                {"value": 18446744073709551615u64}
            ]
        }),
    );

    assert!(output.contains("Value uint64 `json:\"value\"`"), "{output}");
}

#[test]
fn renders_go_custom_json_methods_for_keys_go_tags_cannot_express() {
    let output = rendered("go", "Root", json!({"-": 1, "a,b": 2, "ok": 3}));

    assert!(output.starts_with("package models\n\nimport \"encoding/json\"\n\n"));
    assert!(output.contains("\tField int64 `json:\"-\"`\n"));
    assert!(output.contains("\tAB int64 `json:\"-\"`\n"));
    assert!(output.contains("\tOk int64 `json:\"ok\"`\n"));
    assert!(output.contains("func (value *Root) UnmarshalJSON(data []byte) error {"));
    assert!(output.contains("func (value Root) MarshalJSON() ([]byte, error) {"));
}

#[test]
fn renders_go_root_arrays_as_type_aliases() {
    let output = rendered("go", "Root", json!([{"id": 1}, {"id": 2, "name": "Neko"}]));

    assert_eq!(
        output,
        concat!(
            "package models\n\n",
            "type RootItem struct {\n",
            "\tId int64 `json:\"id\"`\n",
            "\tName *string `json:\"name\"`\n",
            "}\n\n",
            "type Root = []RootItem\n"
        )
    );
}
