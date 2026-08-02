use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn init_project(dir: &Path, name: &str) {
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(dir)
        .env_remove("MAX_CONFIG_FILE")
        .args(["init", name])
        .assert()
        .success();
}

#[test]
fn cmd_add_creates_command_files() {
    let tmp = tempfile::tempdir().unwrap();
    init_project(tmp.path(), "mycli");
    let project = tmp.path().join("mycli");

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "hello", "--desc", "Say hello"])
        .assert()
        .success();

    assert!(project.join("src/commands/hello.rs").exists());

    let mod_rs = fs::read_to_string(project.join("src/commands/mod.rs")).unwrap();
    assert!(mod_rs.contains("pub mod hello;"));

    let cli_rs = fs::read_to_string(project.join("src/cli.rs")).unwrap();
    assert!(cli_rs.contains("Hello(Hello),"));
    assert!(cli_rs.contains("pub struct Hello"));

    let main_rs = fs::read_to_string(project.join("src/main.rs")).unwrap();
    assert!(main_rs.contains("Commands::Hello(args)"));
}

#[test]
fn cmd_add_uses_leaf_segment() {
    let tmp = tempfile::tempdir().unwrap();
    init_project(tmp.path(), "mycli");
    let project = tmp.path().join("mycli");

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "admin.users.list"])
        .assert()
        .success();

    assert!(project.join("src/commands/list.rs").exists());
    assert!(!project.join("src/commands/admin.rs").exists());
    let cli_rs = fs::read_to_string(project.join("src/cli.rs")).unwrap();
    assert!(cli_rs.contains("List(List),"));
}

#[test]
fn cmd_add_duplicate_errors() {
    let tmp = tempfile::tempdir().unwrap();
    init_project(tmp.path(), "mycli");
    let project = tmp.path().join("mycli");

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "greet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn cmd_add_missing_marker_errors() {
    let tmp = tempfile::tempdir().unwrap();
    init_project(tmp.path(), "mycli");
    let project = tmp.path().join("mycli");

    let cli_rs_path = project.join("src/cli.rs");
    let cli_rs = fs::read_to_string(&cli_rs_path).unwrap();
    fs::write(&cli_rs_path, cli_rs.replace("// __CMD_ENUM_MARKER__", "")).unwrap();

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("marker"));
}

#[test]
fn cmd_add_not_a_project_errors() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(tmp.path())
        .args(["cmd", "add", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("init"));
}

#[test]
fn cmd_add_invalid_name_errors() {
    let tmp = tempfile::tempdir().unwrap();
    init_project(tmp.path(), "mycli");
    let project = tmp.path().join("mycli");

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "2bad-name"])
        .assert()
        .failure();
}
