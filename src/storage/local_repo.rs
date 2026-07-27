use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::NoteRepository;
use crate::core::validators::validate_slug;
use crate::storage::frontmatter::{Frontmatter, build_note_content, parse_frontmatter_and_body};

#[derive(Debug, Clone)]
pub struct LocalMarkdownRepo {
    notes_dir: PathBuf,
}

impl Default for LocalMarkdownRepo {
    fn default() -> Self {
        Self::new("notes")
    }
}

impl LocalMarkdownRepo {
    pub fn new<P: AsRef<Path>>(notes_dir: P) -> Self {
        Self {
            notes_dir: notes_dir.as_ref().to_path_buf(),
        }
    }

    pub fn note_path(&self, slug: &str) -> Result<PathBuf, CoreError> {
        let slug = validate_slug(slug)?;
        Ok(self.notes_dir.join(format!("{}.md", slug)))
    }

    fn ensure_notes_dir(&self) -> Result<(), CoreError> {
        if !self.notes_dir.exists() {
            fs::create_dir_all(&self.notes_dir)?;
        }
        Ok(())
    }
}

impl NoteRepository for LocalMarkdownRepo {
    fn note_exists(&self, slug: &str) -> Result<bool, CoreError> {
        Ok(self.note_path(slug)?.exists())
    }

    fn save_note(&self, note: &Note) -> Result<(), CoreError> {
        let note_path = self.note_path(&note.slug)?;
        self.ensure_notes_dir()?;

        let frontmatter = Frontmatter {
            title: note.title.clone(),
            date: note.date.clone(),
            tags: note.tags.clone(),
            updated_at: note.updated_at.clone(),
        };

        let content = build_note_content(&frontmatter, &note.body)?;
        fs::write(note_path, content)?;

        Ok(())
    }

    fn read_note(&self, slug: &str) -> Result<Note, CoreError> {
        let content = self.read_raw_note(slug)?;
        let (fm, body) = parse_frontmatter_and_body(&content)?;

        Ok(Note {
            slug: slug.to_string(),
            title: fm.title,
            date: fm.date,
            tags: fm.tags,
            updated_at: fm.updated_at,
            body,
        })
    }

    fn ensure_note_exists(&self, slug: &str) -> Result<(), CoreError> {
        let path = self.note_path(slug)?;
        if !path.exists() {
            return Err(CoreError::NoteNotFound(path.display().to_string()));
        }

        Ok(())
    }

    fn list_notes(&self) -> Result<Vec<Note>, CoreError> {
        let mut notes = Vec::new();

        for slug in self.list_note_slugs()? {
            notes.push(self.read_note(&slug)?);
        }

        Ok(notes)
    }

    fn list_note_slugs(&self) -> Result<Vec<String>, CoreError> {
        if !self.notes_dir.exists() {
            return Ok(Vec::new());
        }

        let mut slugs = fs::read_dir(&self.notes_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();

        slugs.sort();
        Ok(slugs)
    }

    fn delete_note(&self, slug: &str) -> Result<(), CoreError> {
        let path = self.note_path(slug)?;

        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                Err(CoreError::NoteNotFound(String::new()))
            }
            Err(e) => Err(CoreError::Io(e)),
        }
    }

    fn read_raw_note(&self, slug: &str) -> Result<String, CoreError> {
        self.ensure_note_exists(slug)?;

        let path = self.note_path(slug)?;
        let raw = fs::read_to_string(path)?;

        Ok(raw)
    }

    // fn write_raw_note(&self, slug: &str, content: &str) -> Result<(), CoreError> {
    //     self.ensure_note_exists(slug)?;

    //     let path = self.note_path(slug);
    //     fs::write(path, content)?;
    //     Ok(())
    // }
}
