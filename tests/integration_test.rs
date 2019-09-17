use assert_cmd::crate_name;
use assert_cmd::prelude::*;
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

// TODO: write output with NFC normalization
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
        .buffer(" {\"abc\": 1}\n{\"arthur\"      :\"pomme\", \"1\":1   }\n{\"command_name\":\"achat groupé de pommes\",\"quantity\":123456780,\"commentary\":\"Mangez des pommes !\",\"detail\" :  { \"client\" :\"Jacques Chirac\", \"cash\":   true,\"random_numbers\": [1, 0, 0.1, -1, \"fake it's not a number]\"]}}")
        .assert()
        .success()
        .stdout(predicate::function(|stdout: &str| {
            let output = json_stream_from_str(stdout);
            dbg!(&expected_output);
            dbg!(&output);
            output == expected_output
        }));
}
