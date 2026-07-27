use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::core::errors::CoreError;
use crate::core::note::{Note, NoteDate};
use crate::core::repository::{
    NoteCollection, NoteLoadIssue, NoteRepository, TrashCollection, TrashedNote,
};
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

    fn trash_dir(&self) -> PathBuf {
        self.notes_dir.join(".trash")
    }

    fn validate_trash_id<'a>(&self, id: &'a str) -> Result<&'a str, CoreError> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(CoreError::InvalidTrashId(id.to_string()));
        }
        Ok(id)
    }

    fn trash_paths(&self, id: &str) -> Result<(PathBuf, PathBuf), CoreError> {
        let id = self.validate_trash_id(id)?;
        Ok((
            self.trash_dir().join(format!("{id}.md")),
            self.trash_dir().join(format!("{id}.toml")),
        ))
    }

    fn available_trash_paths(
        &self,
        slug: &str,
        stamp: &str,
    ) -> Result<(String, PathBuf, PathBuf), CoreError> {
        for suffix in 0_u64..=u64::MAX {
            let candidate_id: String = if suffix == 0 {
                format!("{slug}--{stamp}")
            } else {
                format!("{slug}--{stamp}-{suffix}")
            };

            let (note_path, metadata_path): (PathBuf, PathBuf) =
                self.trash_paths(candidate_id.as_str())?;
            if !note_path.exists() && !metadata_path.exists() {
                return Ok((candidate_id, note_path, metadata_path));
            }
        }

        Err(CoreError::Io(std::io::Error::other(
            "trash entry suffix space exhausted",
        )))
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
            date: note.date.to_string(),
            tags: note.tags.clone(),
            updated_at: note.updated_at.to_string(),
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
            date: parse_note_date("date", &fm.date)?,
            tags: fm.tags,
            updated_at: parse_note_date("updated_at", &fm.updated_at)?,
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

    fn list_notes(&self) -> Result<NoteCollection, CoreError> {
        let mut collection = NoteCollection::default();

        for slug in self.list_note_slugs()? {
            match self.read_note(&slug) {
                Ok(note) => collection.notes.push(note),
                Err(error) => collection.issues.push(NoteLoadIssue {
                    slug,
                    message: error.to_string(),
                }),
            }
        }

        Ok(collection)
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

    fn trash_note(&self, slug: &str) -> Result<TrashedNote, CoreError> {
        let source = self.note_path(slug)?;
        if !source.is_file() {
            return Err(CoreError::NoteNotFound(source.display().to_string()));
        }
        fs::create_dir_all(self.trash_dir())?;
        let stamp = chrono::Local::now().format("%Y%m%dT%H%M%S%f").to_string();
        let (id, note_path, metadata_path) = self.available_trash_paths(slug, &stamp)?;
        let entry = TrashedNote {
            id,
            original_slug: slug.to_string(),
            deleted_at: chrono::Local::now().to_rfc3339(),
        };
        let metadata =
            toml::to_string(&entry).map_err(|error| std::io::Error::other(error.to_string()))?;
        fs::write(&metadata_path, metadata)?;
        if let Err(error) = fs::rename(&source, &note_path) {
            let _ = fs::remove_file(metadata_path);
            return Err(CoreError::Io(error));
        }
        Ok(entry)
    }

    fn list_trash(&self) -> Result<TrashCollection, CoreError> {
        let mut collection = TrashCollection::default();
        if !self.trash_dir().exists() {
            return Ok(collection);
        }
        for entry in fs::read_dir(self.trash_dir())? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("toml") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string();
            let result = fs::read_to_string(&path)
                .map_err(CoreError::Io)
                .and_then(|raw| {
                    toml::from_str::<TrashedNote>(&raw)
                        .map_err(|error| CoreError::InvalidFrontmatter(error.to_string()))
                })
                .and_then(|metadata| {
                    if metadata.id != id {
                        return Err(CoreError::InvalidTrashId(metadata.id));
                    }
                    if self.trash_paths(&metadata.id)?.0.is_file() {
                        Ok(metadata)
                    } else {
                        Err(CoreError::TrashEntryNotFound(id.clone()))
                    }
                });
            match result {
                Ok(metadata) => collection.entries.push(metadata),
                Err(error) => collection.issues.push(NoteLoadIssue {
                    slug: id,
                    message: error.to_string(),
                }),
            }
        }
        collection.entries.sort_by(|a, b| {
            b.deleted_at
                .cmp(&a.deleted_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(collection)
    }

    fn restore_trash(&self, id: &str) -> Result<Note, CoreError> {
        let (trash_note, metadata_path) = self.trash_paths(id)?;
        if !trash_note.is_file() || !metadata_path.is_file() {
            return Err(CoreError::TrashEntryNotFound(id.to_string()));
        }
        let metadata: TrashedNote = toml::from_str(&fs::read_to_string(&metadata_path)?)
            .map_err(|error| CoreError::InvalidFrontmatter(error.to_string()))?;
        let destination = self.note_path(&metadata.original_slug)?;
        if destination.exists() {
            return Err(CoreError::RestoreCollision {
                slug: metadata.original_slug,
            });
        }
        self.ensure_notes_dir()?;
        fs::rename(&trash_note, &destination)?;
        fs::remove_file(metadata_path)?;
        self.read_note(&metadata.original_slug)
    }

    fn purge_trash(&self, id: &str) -> Result<(), CoreError> {
        let (note, metadata) = self.trash_paths(id)?;
        if !note.exists() && !metadata.exists() {
            return Err(CoreError::TrashEntryNotFound(id.to_string()));
        }
        match fs::remove_file(note) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        match fs::remove_file(metadata) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn read_raw_note(&self, slug: &str) -> Result<String, CoreError> {
        self.ensure_note_exists(slug)?;

        let path = self.note_path(slug)?;
        let raw = fs::read_to_string(path)?;

        Ok(raw)
    }
}

fn parse_note_date(field: &'static str, value: &str) -> Result<NoteDate, CoreError> {
    value.parse().map_err(|_| CoreError::InvalidDate {
        field,
        value: value.to_string(),
    })
}
