use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io;

use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::{NoteCollection, NoteLoadIssue, NoteRepository};

#[derive(Debug, Default)]
pub struct InMemoryNoteRepository {
    notes: RefCell<BTreeMap<String, Note>>,
    list_issues: RefCell<Vec<NoteLoadIssue>>,
    read_calls: Cell<usize>,
    save_calls: Cell<usize>,
    fail_exists: Cell<bool>,
    fail_reads: Cell<bool>,
    fail_saves: Cell<bool>,
}

impl InMemoryNoteRepository {
    pub fn insert(&self, note: Note) {
        self.notes.borrow_mut().insert(note.slug.clone(), note);
    }

    pub fn get(&self, slug: &str) -> Option<Note> {
        self.notes.borrow().get(slug).cloned()
    }

    pub fn note_count(&self) -> usize {
        self.notes.borrow().len()
    }

    pub fn read_calls(&self) -> usize {
        self.read_calls.get()
    }

    pub fn save_calls(&self) -> usize {
        self.save_calls.get()
    }

    pub fn fail_exists(&self) {
        self.fail_exists.set(true);
    }

    pub fn fail_reads(&self) {
        self.fail_reads.set(true);
    }

    pub fn fail_saves(&self) {
        self.fail_saves.set(true);
    }

    pub fn add_list_issue(&self, issue: NoteLoadIssue) {
        self.list_issues.borrow_mut().push(issue);
    }

    fn injected_io_error(operation: &str) -> CoreError {
        CoreError::Io(io::Error::other(format!("injected {operation} failure")))
    }
}

impl NoteRepository for InMemoryNoteRepository {
    fn note_exists(&self, slug: &str) -> Result<bool, CoreError> {
        if self.fail_exists.get() {
            return Err(Self::injected_io_error("exists"));
        }

        Ok(self.notes.borrow().contains_key(slug))
    }

    fn save_note(&self, note: &Note) -> Result<(), CoreError> {
        self.save_calls.set(self.save_calls.get() + 1);
        if self.fail_saves.get() {
            return Err(Self::injected_io_error("save"));
        }

        self.notes
            .borrow_mut()
            .insert(note.slug.clone(), note.clone());
        Ok(())
    }

    fn read_note(&self, slug: &str) -> Result<Note, CoreError> {
        self.read_calls.set(self.read_calls.get() + 1);
        if self.fail_reads.get() {
            return Err(Self::injected_io_error("read"));
        }

        self.notes
            .borrow()
            .get(slug)
            .cloned()
            .ok_or_else(|| CoreError::NoteNotFound(slug.to_string()))
    }

    fn ensure_note_exists(&self, slug: &str) -> Result<(), CoreError> {
        if self.notes.borrow().contains_key(slug) {
            Ok(())
        } else {
            Err(CoreError::NoteNotFound(slug.to_string()))
        }
    }

    fn list_notes(&self) -> Result<NoteCollection, CoreError> {
        Ok(NoteCollection {
            notes: self.notes.borrow().values().cloned().collect(),
            issues: self.list_issues.borrow().clone(),
        })
    }

    fn list_note_slugs(&self) -> Result<Vec<String>, CoreError> {
        Ok(self.notes.borrow().keys().cloned().collect())
    }

    fn read_raw_note(&self, slug: &str) -> Result<String, CoreError> {
        self.read_note(slug).map(|note| note.body)
    }
}

pub fn note(slug: &str) -> Note {
    Note {
        slug: slug.to_string(),
        title: "Test Note".to_string(),
        date: "2026-07-27".parse().unwrap(),
        tags: vec![],
        updated_at: "2026-07-27".parse().unwrap(),
        body: "Body".to_string(),
    }
}
