use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn max_config_file_env_override() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("override.json"), "{}").unwrap();
    let expected = dir.path().join("override.json");
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(dir.path())
        .env("MAX_CONFIG_FILE", &expected)
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            expected.to_string_lossy().as_ref(),
        ));
}

#[test]
fn malformed_local_config_warns() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("max.json"), "{ not json").unwrap();
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(dir.path())
        .env_remove("MAX_CONFIG_FILE")
        .args(["config", "path"])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning: failed to parse"));
}
