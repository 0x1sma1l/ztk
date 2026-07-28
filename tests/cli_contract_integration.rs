use std::process::{Command, Output};

fn ztk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(args)
        .output()
        .expect("failed to execute ztk")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn help_lists_the_complete_cli_v1_command_set() {
    let output = ztk(&["--help"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let output = stdout(&output);
    for command in [
        "new", "list", "edit", "update", "view", "search", "lint", "stats", "delete", "trash",
        "purge", "tui",
    ] {
        assert!(output.contains(command), "missing command: {command}");
    }
}

#[test]
fn version_reports_the_package_version() {
    let output = ztk(&["--version"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        format!("ztk {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(stderr(&output).is_empty());
}

#[test]
fn clap_contract_errors_use_exit_two_and_stderr() {
    for args in [&["unknown"][..], &["view"][..], &["new"][..]] {
        let output = ztk(args);

        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(stdout(&output).is_empty(), "args: {args:?}");
        assert!(stderr(&output).contains("Usage:"), "args: {args:?}");
    }
}

#[test]
fn runtime_errors_use_exit_one_and_stderr() {
    let temporary = tempfile::TempDir::new().expect("failed to create temporary directory");
    let output = Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["view", "missing"])
        .current_dir(temporary.path())
        .env("XDG_DATA_HOME", temporary.path().join("data"))
        .env_remove("ZTK_NOTES_DIR")
        .env_remove("ZTK_CONFIG")
        .output()
        .expect("failed to execute ztk");

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("Note not found"));
}
