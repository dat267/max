use assert_cmd::Command;

#[test]
fn end_to_end_init_add_build_run() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("max")
        .unwrap()
        .current_dir(tmp.path())
        .env_remove("MAX_CONFIG_FILE")
        .args(["init", "mycli"])
        .assert()
        .success();
    let project = tmp.path().join("mycli");

    Command::cargo_bin("max")
        .unwrap()
        .current_dir(&project)
        .args(["cmd", "add", "hello"])
        .assert()
        .success();

    // cargo test builds the scaffold AND runs the template unit tests.
    let test = std::process::Command::new("cargo")
        .arg("test")
        .current_dir(&project)
        .output()
        .expect("cargo test should run in the generated project");
    assert!(
        test.status.success(),
        "cargo test in generated project failed:\n{}",
        String::from_utf8_lossy(&test.stderr)
    );

    // cargo build guarantees the target/debug/<name> binary artifact exists.
    let build = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(&project)
        .output()
        .expect("cargo build should run in the generated project");
    assert!(
        build.status.success(),
        "cargo build in generated project failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let bin = project.join("target/debug/mycli");
    let out = std::process::Command::new(&bin)
        .arg("hello")
        .output()
        .expect("generated binary should run");
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("TODO: implement Hello command"),
        "unexpected output: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
