use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn assert_file_exists(base: &Path, rel: &str) {
    assert!(base.join(rel).exists(), "expected {rel} to exist");
}

#[test]
fn init_creates_project_structure() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(tmp.path())
        .env_remove("MAX_CONFIG_FILE")
        .args(["init", "mycli"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created project \"mycli\""));

    let project = tmp.path().join("mycli");
    for f in [
        "Cargo.toml",
        ".gitignore",
        ".github/workflows/release.yml",
        "src/main.rs",
        "src/cli.rs",
        "src/config.rs",
        "src/commands/mod.rs",
        "src/commands/greet.rs",
        "src/commands/config.rs",
    ] {
        assert_file_exists(&project, f);
    }

    let cli = fs::read_to_string(project.join("src/cli.rs")).unwrap();
    assert!(cli.contains("mycli"), "project name should be substituted");
}

#[test]
fn init_refuses_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("mycli");
    fs::create_dir_all(&project).unwrap();

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(tmp.path())
        .args(["init", "mycli"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn init_rejects_invalid_names() {
    let tmp = tempfile::tempdir().unwrap();
    for bad in ["../escape", "a/b", "a\\b"] {
        Command::cargo_bin("max")
            .unwrap()
            .current_dir(tmp.path())
            .args(["init", bad])
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid project name"));
    }
}
