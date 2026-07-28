use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn ztk(root: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(args)
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .current_dir(root.path())
        .output()
        .unwrap()
}

fn write_note(root: &TempDir, slug: &str, body: &str) {
    let notes = root.path().join("notes");
    fs::create_dir_all(&notes).unwrap();
    fs::write(
        notes.join(format!("{slug}.md")),
        format!("---\ntitle: {slug}\ndate: 2026-07-27\ntags: []\nupdated_at: 2026-07-27\n---\n\n{body}\n"),
    ).unwrap();
}

fn trash_ids(root: &TempDir) -> Vec<String> {
    let mut ids = fs::read_dir(root.path().join("notes/.trash"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("toml"))
        .map(|entry| {
            entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn delete_list_and_restore_round_trip() {
    let root = TempDir::new().unwrap();
    write_note(&root, "recover-me", "valuable body");
    assert!(
        ztk(&root, &["delete", "recover-me", "--force"])
            .status
            .success()
    );
    let id = trash_ids(&root).pop().unwrap();

    let listed = ztk(&root, &["trash", "list"]);
    assert!(listed.status.success());
    assert!(text(&listed.stdout).contains(&id));
    assert!(text(&listed.stdout).contains("recover-me"));

    let restored = ztk(&root, &["trash", "restore", &id]);
    assert!(restored.status.success(), "{}", text(&restored.stderr));
    let content = fs::read_to_string(root.path().join("notes/recover-me.md")).unwrap();
    assert!(content.contains("valuable body"));
    assert!(trash_ids(&root).is_empty());
}

#[test]
fn restore_collision_preserves_live_and_trashed_copies() {
    let root = TempDir::new().unwrap();
    write_note(&root, "collision", "trashed copy");
    assert!(
        ztk(&root, &["delete", "collision", "--force"])
            .status
            .success()
    );
    let id = trash_ids(&root).pop().unwrap();
    write_note(&root, "collision", "live copy");

    let output = ztk(&root, &["trash", "restore", &id]);

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("already exists"));
    assert!(
        fs::read_to_string(root.path().join("notes/collision.md"))
            .unwrap()
            .contains("live copy")
    );
    assert_eq!(trash_ids(&root), [id]);
}

#[test]
fn repeated_deletions_get_unique_entries() {
    let root = TempDir::new().unwrap();
    write_note(&root, "repeat", "first");
    assert!(
        ztk(&root, &["delete", "repeat", "--force"])
            .status
            .success()
    );
    write_note(&root, "repeat", "second");
    assert!(
        ztk(&root, &["delete", "repeat", "--force"])
            .status
            .success()
    );

    let ids = trash_ids(&root);
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn purge_one_refuses_without_an_interactive_confirmation() {
    let root = TempDir::new().unwrap();
    write_note(&root, "purge-me", "body");
    assert!(
        ztk(&root, &["delete", "purge-me", "--force"])
            .status
            .success()
    );
    let id = trash_ids(&root).pop().unwrap();

    let refused = ztk(&root, &["purge", &id]);

    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("interactive terminal"));
    assert_eq!(trash_ids(&root), [id]);
}

#[test]
fn purge_all_refuses_without_an_interactive_confirmation() {
    let root = TempDir::new().unwrap();
    write_note(&root, "keep-me", "body");
    assert!(
        ztk(&root, &["delete", "keep-me", "--force"])
            .status
            .success()
    );

    let refused = ztk(&root, &["purge", "--all"]);

    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("interactive terminal"));
    assert_eq!(trash_ids(&root).len(), 1);
}

#[test]
fn list_reports_malformed_metadata_and_invalid_ids_are_rejected() {
    let root = TempDir::new().unwrap();
    write_note(&root, "healthy", "body");
    assert!(
        ztk(&root, &["delete", "healthy", "--force"])
            .status
            .success()
    );
    fs::write(root.path().join("notes/.trash/broken.toml"), "invalid").unwrap();

    let listed = ztk(&root, &["trash", "list"]);
    assert!(listed.status.success());
    assert!(text(&listed.stdout).contains("healthy"));
    assert!(text(&listed.stderr).contains("broken"));

    let traversal = ztk(&root, &["trash", "restore", "../outside"]);
    assert_eq!(traversal.status.code(), Some(1));
    assert!(text(&traversal.stderr).contains("Invalid trash ID"));
}
