use jsontolang::plugins;
use jsontolang::schema::infer_document;
use serde_json::json;
use std::fs;
use std::io::ErrorKind;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn command_exists_reports_missing_binary_as_unavailable() {
    assert!(!command_exists("__jsontolang_missing_binary__"));
}

#[test]
fn renders_typescript_nested_interfaces() {
    let value = json!({"user": {"name": "Neko"}});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("typescript")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("export interface Root {"));
    assert!(output.contains("user: RootUser;"));
    assert!(output.contains("export interface RootUser {"));
    assert!(output.contains("name: string;"));
}

#[test]
fn renders_typescript_optional_properties() {
    let value = json!({"items": [{"id": 1, "name": "A"}, {"id": 2}]});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("typescript")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("export interface RootItemsItem {"));
    assert!(output.contains("name?: string;"));
}

#[test]
fn renders_typescript_non_identifier_property_keys() {
    let value = json!({"display-name": "Neko", "123flag": true, "two words": "ok"});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("typescript")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("\"display-name\": string;"));
    assert!(output.contains("\"123flag\": boolean;"));
    assert!(output.contains("\"two words\": string;"));
}

#[test]
fn sanitizes_invalid_root_names_for_typescript_declarations() {
    let value = json!({"child": {"name": "Neko"}});
    let document = infer_document("123 root-name", &value).unwrap();
    let output = plugins::lookup_by_key("typescript")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("export interface _123RootName {"));
    assert!(output.contains("child: _123RootNameChild;"));
    assert!(output.contains("export interface _123RootNameChild {"));
}

#[test]
fn renders_typescript_root_arrays_as_exported_type_aliases() {
    let value = json!([{"id": 1}, {"id": 2, "name": "Neko"}]);
    let document = infer_document("123 root-name", &value).unwrap();
    let output = plugins::lookup_by_key("typescript")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("export interface _123RootNameItem {"));
    assert!(output.contains("name?: string;"));
    assert!(output.contains("export type _123RootName = _123RootNameItem[];"));
}

#[test]
fn renders_rust_structs_with_serde_renames() {
    let value = json!({
        "display-name": "Neko",
        "items": [{"id": 1}, {"id": 2, "extra": null}],
        "child": {"two words": true}
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!([{"id": 1}, {"id": 2, "name": "Neko"}]);
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("pub struct RootItem {"));
    assert!(output.contains("pub name: Option<String>,"));
    assert!(output.contains("pub type Root = Vec<RootItem>;"));
}

#[test]
fn renders_rust_colliding_and_keyword_field_names_deterministically() {
    let value = json!({
        "display name": "A",
        "display-name": "B",
        "type": 1,
        "type!": 2
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({
        "samples": [
            {"value": 9223372036854775808u64},
            {"value": 18446744073709551615u64}
        ]
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("pub value: u64,"), "{output}");
}

#[test]
fn renders_rust_reserved_root_type_names_safely() {
    let value = json!({"child": {"name": "Neko"}});
    let document = infer_document("self", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({"type": {"name": "Neko"}});
    let document = infer_document("self", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({
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
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({
        "display name": "A",
        "display-name": "B"
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({"tick`key": 1});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({
        "path\\name": 1,
        "quote\"name": 2,
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

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
    let value = json!({
        "samples": [
            {"value": 9223372036854775808u64},
            {"value": 18446744073709551615u64}
        ]
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

    assert!(output.contains("Value uint64 `json:\"value\"`"), "{output}");
}

#[test]
fn generated_go_output_round_trips_dash_and_comma_keys() {
    if !command_exists("go") {
        return;
    }

    let value = json!({"-": 1, "a,b": 2, "ok": 3});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();
    let temp = tempdir().unwrap();

    fs::write(
        temp.path().join("go.mod"),
        concat!("module generatedmodels\n\n", "go 1.20\n"),
    )
    .unwrap();
    fs::write(temp.path().join("models.go"), output).unwrap();
    fs::write(
        temp.path().join("models_test.go"),
        concat!(
            "package models\n\n",
            "import (\n",
            "\t\"encoding/json\"\n",
            "\t\"testing\"\n",
            ")\n\n",
            "func TestRoundTripsSpecialJSONKeys(t *testing.T) {\n",
            "\tinput := []byte(`{\"-\":1,\"a,b\":2,\"ok\":3}`)\n",
            "\tvar value Root\n",
            "\tif err := json.Unmarshal(input, &value); err != nil {\n",
            "\t\tt.Fatalf(\"unmarshal failed: %v\", err)\n",
            "\t}\n",
            "\tif value.Field != 1 {\n",
            "\t\tt.Fatalf(\"unexpected dash field: %v\", value.Field)\n",
            "\t}\n",
            "\tif value.AB != 2 {\n",
            "\t\tt.Fatalf(\"unexpected comma field: %v\", value.AB)\n",
            "\t}\n",
            "\tif value.Ok != 3 {\n",
            "\t\tt.Fatalf(\"unexpected ok field: %v\", value.Ok)\n",
            "\t}\n",
            "\tencoded, err := json.Marshal(value)\n",
            "\tif err != nil {\n",
            "\t\tt.Fatalf(\"marshal failed: %v\", err)\n",
            "\t}\n",
            "\tvar decoded map[string]int\n",
            "\tif err := json.Unmarshal(encoded, &decoded); err != nil {\n",
            "\t\tt.Fatalf(\"decode failed: %v\", err)\n",
            "\t}\n",
            "\tif decoded[\"-\"] != 1 {\n",
            "\t\tt.Fatalf(\"unexpected encoded dash field: %v\", decoded)\n",
            "\t}\n",
            "\tif decoded[\"a,b\"] != 2 {\n",
            "\t\tt.Fatalf(\"unexpected encoded comma field: %v\", decoded)\n",
            "\t}\n",
            "\tif decoded[\"ok\"] != 3 {\n",
            "\t\tt.Fatalf(\"unexpected encoded ok field: %v\", decoded)\n",
            "\t}\n",
            "}\n"
        ),
    )
    .unwrap();

    let result = Command::new("go")
        .arg("test")
        .arg("./...")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "go test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn generated_go_output_round_trips_control_character_keys() {
    if !command_exists("go") {
        return;
    }

    let value = json!({"line\nfeed": 1, "tab\tkey": 2, "ok": 3});
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();
    let temp = tempdir().unwrap();

    fs::write(
        temp.path().join("go.mod"),
        concat!("module generatedmodels\n\n", "go 1.20\n"),
    )
    .unwrap();
    fs::write(temp.path().join("models.go"), output).unwrap();
    fs::write(
        temp.path().join("models_test.go"),
        concat!(
            "package models\n\n",
            "import (\n",
            "\t\"encoding/json\"\n",
            "\t\"testing\"\n",
            ")\n\n",
            "func TestRoundTripsControlCharacterJSONKeys(t *testing.T) {\n",
            "\tinput := []byte(\"{\\\"line\\\\nfeed\\\":1,\\\"tab\\\\tkey\\\":2,\\\"ok\\\":3}\")\n",
            "\tvar value Root\n",
            "\tif err := json.Unmarshal(input, &value); err != nil {\n",
            "\t\tt.Fatalf(\"unmarshal failed: %v\", err)\n",
            "\t}\n",
            "\tif value.LineFeed != 1 {\n",
            "\t\tt.Fatalf(\"unexpected newline field: %v\", value.LineFeed)\n",
            "\t}\n",
            "\tif value.TabKey != 2 {\n",
            "\t\tt.Fatalf(\"unexpected tab field: %v\", value.TabKey)\n",
            "\t}\n",
            "\tif value.Ok != 3 {\n",
            "\t\tt.Fatalf(\"unexpected ok field: %v\", value.Ok)\n",
            "\t}\n",
            "\tencoded, err := json.Marshal(value)\n",
            "\tif err != nil {\n",
            "\t\tt.Fatalf(\"marshal failed: %v\", err)\n",
            "\t}\n",
            "\tvar decoded map[string]int\n",
            "\tif err := json.Unmarshal(encoded, &decoded); err != nil {\n",
            "\t\tt.Fatalf(\"decode failed: %v\", err)\n",
            "\t}\n",
            "\tif decoded[\"line\\nfeed\"] != 1 {\n",
            "\t\tt.Fatalf(\"unexpected encoded newline field: %v\", decoded)\n",
            "\t}\n",
            "\tif decoded[\"tab\\tkey\"] != 2 {\n",
            "\t\tt.Fatalf(\"unexpected encoded tab field: %v\", decoded)\n",
            "\t}\n",
            "\tif decoded[\"ok\"] != 3 {\n",
            "\t\tt.Fatalf(\"unexpected encoded ok field: %v\", decoded)\n",
            "\t}\n",
            "}\n"
        ),
    )
    .unwrap();

    let result = Command::new("go")
        .arg("test")
        .arg("./...")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "go test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn renders_go_root_arrays_as_type_aliases() {
    let value = json!([{"id": 1}, {"id": 2, "name": "Neko"}]);
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();

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

#[test]
fn representative_rust_output_passes_cargo_check() {
    let value = json!({
        "type": {"name": "Neko"},
        "items": [{"id": 1}, {"id": 2, "extra": null}]
    });
    let document = infer_document("self", &value).unwrap();
    let output = plugins::lookup_by_key("rust")
        .unwrap()
        .render(&document)
        .unwrap();
    let temp = tempdir().unwrap();
    let src_dir = temp.path().join("src");

    fs::create_dir(&src_dir).unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"generated_models\"\n",
            "version = \"0.1.0\"\n",
            "edition = \"2024\"\n\n",
            "[dependencies]\n",
            "serde = { version = \"1.0\", features = [\"derive\"] }\n",
            "serde_json = \"1.0\"\n"
        ),
    )
    .unwrap();
    fs::write(src_dir.join("lib.rs"), output).unwrap();

    let result = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "cargo check failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn representative_go_output_passes_gofmt() {
    let value = json!({
        "tick`key": 1,
        "items": [
            {"id": 1, "aliases": ["A"], "extra": null},
            {"id": 2}
        ]
    });
    let document = infer_document("Root", &value).unwrap();
    let output = plugins::lookup_by_key("go")
        .unwrap()
        .render(&document)
        .unwrap();
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("models.go");

    if !command_exists("gofmt") {
        return;
    }

    fs::write(&file_path, output).unwrap();

    let result = Command::new("gofmt").arg(&file_path).output().unwrap();

    assert!(
        result.status.success(),
        "gofmt failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

fn command_exists(command: &str) -> bool {
    match Command::new(command).arg("--version").output() {
        Ok(_) => true,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(_) => false,
    }
}
