use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn rejects_missing_input_source() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "typescript"])
        .assert()
        .failure()
        .stderr(contains(
            "the following required arguments were not provided",
        ))
        .stderr(contains("<--file <FILE>|--stdin|--json <JSON>>"));
}

#[test]
fn rejects_multiple_input_sources() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "typescript", "--stdin", "--json", "{}"])
        .assert()
        .failure()
        .stderr(contains(
            "the argument '--stdin' cannot be used with '--json <JSON>'",
        ))
        .stderr(contains("<--file <FILE>|--stdin|--json <JSON>>"));
}

#[test]
fn help_advertises_exactly_one_input_source_is_required() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--help"])
        .assert()
        .success()
        .stdout(contains("--file <FILE>|--stdin|--json <JSON>"));
}

#[test]
fn valid_single_source_renders_typescript_output() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "typescript", "--json", r#"{"name":"Neko"}"#])
        .assert()
        .success()
        .stdout("export interface Root {\n  name: string;\n}\n\n");
}

#[test]
fn reads_json_from_file_flag_and_renders_typescript_output() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{{\"name\":\"Neko\"}}").unwrap();

    Command::cargo_bin("jsontolang")
        .unwrap()
        .args([
            "--lang",
            "typescript",
            "--file",
            file.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("export interface Root {\n  name: string;\n}\n\n");
}

#[test]
fn reads_json_from_stdin_flag_and_renders_typescript_output() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "typescript", "--stdin"])
        .write_stdin("{\"name\":\"Neko\"}")
        .assert()
        .success()
        .stdout("export interface Root {\n  name: string;\n}\n\n");
}

#[test]
fn invalid_input_file_reports_contextual_error() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args([
            "--lang",
            "typescript",
            "--file",
            "/definitely/missing/input.json",
        ])
        .assert()
        .failure()
        .stderr(contains(
            "failed to read input file `/definitely/missing/input.json`",
        ));
}

#[test]
fn reports_invalid_json() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "typescript", "--json", "not-json"])
        .assert()
        .failure()
        .stderr(contains("invalid JSON input"));
}

#[test]
fn renders_selected_go_output() {
    Command::cargo_bin("jsontolang")
        .unwrap()
        .args(["--lang", "go", "--json", r#"{"name":"Neko"}"#])
        .assert()
        .success()
        .stdout("package models\n\ntype Root struct {\n\tName string `json:\"name\"`\n}\n\n");
}
