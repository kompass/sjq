use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::process::Command;

fn json_stream_from_str(s: &str) -> Vec<Value> {
    let mut acc: Vec<Value> = Vec::new();

    for line in s.lines() {
        acc.push(serde_json::from_str(line).unwrap());
    }

    acc
}

#[test]
fn it_requires_args() {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "error: The following required arguments were not provided:",
        ));
}

#[test]
fn it_prints_usage_when_requested() {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&["--help"])
        .assert()
        .success()
        // Test if the output contains the content of help/query_syntax.txt
        .stdout(predicate::str::contains("EXAMPLES :"));
}

#[test]
fn it_parses_simple_stream() {
    let expected_output: Vec<Value> = vec![
        json!({
            "abc": 1
        }),
        json!({
            "arthur": "pomme",
            "1": 1
        }),
        json!({
            "command_name": "achat groupé de pommes",
            "quantity": 123456780,
            "commentary": "Mangez des pommes !",
            "detail": {
                "client": "Jacques Chirac",
                "cash": true,
                "random_numbers": [1, 0, 0.1, -1, "fake it's not a number]"]
            }
        }),
    ];

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&["."])
        .with_stdin()
        .buffer(" {\"abc\": 1}\n{\"arthur\"      :\"pomme\", \"1\":1   }{\"command_name\":\"achat groupé de pommes\",\"quantity\":123456780,\"commentary\":\"Mangez des pommes !\",\"detail\" :  { \"client\" :\"Jacques Chirac\", \"cash\":   true,\"random_numbers\": [1, 0, 0.1, -1, \"fake it's not a number]\"]}}")
        .assert()
        .success()
        .stdout(predicate::function(|stdout: &str| {
            json_stream_from_str(stdout) == expected_output
        }));
}

#[test]
fn it_accepts_pipes_in_query() {
    let syntaxes = [
        ".abc|mean .",
        ".abc |mean .",
        ".abc| mean .",
        ".abc | mean .",
    ];

    for syntax in syntaxes.iter() {
        Command::cargo_bin(crate_name!())
            .unwrap()
            .args(&[syntax])
            .with_stdin()
            .buffer("{\"abc\": 1}{\"abc\": 2}{\"abc\": -1.1}{\"abc\": 1234}{\"abc\": -34.837}")
            .assert()
            .success()
            .stdout("240.2126\n");
    }
}

#[test]
fn it_selects_correct_field_on_pipelined_stream() {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&[". | mean .a.b"])
        .with_stdin()
        .buffer("{\"a\": {\"b\": 1, \"a\": 10000}}{\"b\": -10000, \"a\": {\"b\": -1.1}}{\"a\": {\"b\": 1234}}{\"a\": {\"b\": 2}}{\"a\": {\"b\": -34.837}}")
        .assert()
        .success()
        .stdout("240.2126\n");
}

#[test]
fn it_outputs_in_new_file_when_requested() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let output_file = temp_dir.child("temp-output.json");

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&["--output", output_file.path().to_str().unwrap(), "."])
        .with_stdin()
        .buffer("{\"test\": true}")
        .assert()
        .success();

    output_file.assert(predicate::path::is_file());
    output_file.assert("{\"test\":true}\n");

    temp_dir.close().unwrap();
}

#[test]
fn it_overwrites_file_when_requested() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let output_file = temp_dir.child("temp-output.json");

    output_file
        .write_str("This sentence has to be overwrited.")
        .unwrap();

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&["--output", output_file.path().to_str().unwrap(), "."])
        .with_stdin()
        .buffer("{\"test\": true}")
        .assert()
        .success();

    output_file.assert(predicate::path::is_file());
    output_file.assert("{\"test\":true}\n");

    temp_dir.close().unwrap();
}

#[test]
fn it_appends_file_when_requested() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let output_file = temp_dir.child("temp-output.json");

    output_file.write_str("This sentence has to stay.").unwrap();

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&[
            "--output",
            output_file.path().to_str().unwrap(),
            "--append",
            ".",
        ])
        .with_stdin()
        .buffer("{\"test\": true}")
        .assert()
        .success();

    output_file.assert(predicate::path::is_file());
    output_file.assert("This sentence has to stay.{\"test\":true}\n");

    temp_dir.close().unwrap();
}

#[test]
fn it_refuses_to_write_to_existing_file_when_requested() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let output_file = temp_dir.child("temp-output.json");

    output_file.touch().unwrap();

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&[
            "--output",
            output_file.path().to_str().unwrap(),
            "--force-new",
            ".",
        ])
        .with_stdin()
        .buffer("{\"test\": true}")
        .assert()
        .failure();

    temp_dir.close().unwrap();
}
