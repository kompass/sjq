use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
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
    let expected_output: Vec<Value> =
        json_stream_from_str("{\"abc\":1}\n{\"arthur\":\"pomme\",\"1\":1}\n");

    Command::cargo_bin(crate_name!())
        .unwrap()
        .args(&["."])
        .with_stdin()
        .buffer(" {\"abc\": 1}\n{\"arthur\"      :\"pomme\", \"1\":1   }")
        .assert()
        .success()
        .stdout(predicate::function(|stdout: &str| {
            json_stream_from_str(stdout) == expected_output
        }));
}
