use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn run(root: &TempDir, args: &[&str], environment: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zet"));
    command
        .args(args)
        .current_dir(root.path())
        .env_remove("ZET_NOTES_DIR")
        .env_remove("ZET_CONFIG");
    for (name, value) in environment {
        command.env(name, value);
    }
    command.output().expect("failed to execute zet")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn explicit_notes_dir_wins_and_supports_spaces_and_missing_directories() {
    let root = TempDir::new().unwrap();
    let explicit = root.path().join("my notes/new repository");
    let environment = root.path().join("environment-notes");
    let config = root.path().join("config.toml");
    fs::write(&config, "notes_dir = 'config-notes'").unwrap();

    let output = run(
        &root,
        &[
            "--notes-dir",
            explicit.to_str().unwrap(),
            "new",
            "Configured Note",
        ],
        &[("ZET_NOTES_DIR", &environment), ("ZET_CONFIG", &config)],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(explicit.join("configured-note.md").exists());
    assert!(!environment.exists());
    assert!(!root.path().join("config-notes").exists());
}

#[test]
fn environment_wins_over_config_and_resolves_from_working_directory() {
    let root = TempDir::new().unwrap();
    let config = root.path().join("settings/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "notes_dir = 'from-config'").unwrap();

    let output = run(
        &root,
        &["new", "Environment Note"],
        &[
            ("ZET_NOTES_DIR", Path::new("from-environment")),
            ("ZET_CONFIG", &config),
        ],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        root.path()
            .join("from-environment/environment-note.md")
            .exists()
    );
    assert!(!config.parent().unwrap().join("from-config").exists());
}

#[test]
fn relative_config_path_is_resolved_beside_the_config_file() {
    let root = TempDir::new().unwrap();
    let config = root.path().join("settings/config.toml");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "notes_dir = 'configured notes'").unwrap();

    let output = run(&root, &["new", "Config Note"], &[("ZET_CONFIG", &config)]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        config
            .parent()
            .unwrap()
            .join("configured notes/config-note.md")
            .exists()
    );
}

#[test]
fn malformed_or_missing_explicit_config_is_actionable() {
    let root = TempDir::new().unwrap();
    let malformed = root.path().join("malformed.toml");
    fs::write(&malformed, "unknown = true").unwrap();

    let invalid = run(&root, &["list"], &[("ZET_CONFIG", &malformed)]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(stderr(&invalid).contains("invalid config file"));
    assert!(stderr(&invalid).contains("malformed.toml"));

    let missing = root.path().join("missing.toml");
    let absent = run(&root, &["list"], &[("ZET_CONFIG", &missing)]);
    assert_eq!(absent.status.code(), Some(1));
    assert!(stderr(&absent).contains("failed to read config file"));
}

#[test]
fn configured_path_that_is_a_file_fails_as_a_repository() {
    let root = TempDir::new().unwrap();
    let file = root.path().join("not-a-directory");
    fs::write(&file, "occupied").unwrap();

    let output = run(
        &root,
        &["--notes-dir", file.to_str().unwrap(), "new", "Cannot Save"],
        &[],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("I/O error"));
    assert_eq!(fs::read_to_string(file).unwrap(), "occupied");
}

#[cfg(unix)]
#[test]
fn read_only_repository_reports_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDir::new().unwrap();
    let notes = root.path().join("read-only");
    fs::create_dir(&notes).unwrap();
    fs::set_permissions(&notes, fs::Permissions::from_mode(0o555)).unwrap();

    let output = run(
        &root,
        &["--notes-dir", notes.to_str().unwrap(), "new", "Cannot Save"],
        &[],
    );

    fs::set_permissions(&notes, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("I/O error"));
    assert!(!notes.join("cannot-save.md").exists());
}
