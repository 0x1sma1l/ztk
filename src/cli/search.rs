#![allow(dead_code)]

use crate::storage::frontmatter::parse_frontmatter_and_body;
use std::{error::Error, fs, path::Path};

pub fn interactive_search() -> Result<(), Box<dyn Error>> {
    let notes_dir = Path::new("notes");

    if !notes_dir.exists() {
        return Err("Notes directory does not exist".into());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(notes_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let content = fs::read_to_string(&path)?;

        if let Ok((frontmatter, _)) = parse_frontmatter_and_body(&content) {
            entries.push(format!("{} => {}", slug, frontmatter.title));
        }
    }

    if entries.is_empty() {
        println!("No notes found.");
    }

    Ok(())
}
