use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::core::errors::CoreError;
use crate::core::repository::{NoteLoadIssue, NoteRepository};

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub slug: String,
    pub title: String,
    pub tags: Vec<String>,
    pub score: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SearchResults {
    pub matches: Vec<SearchMatch>,
    pub issues: Vec<NoteLoadIssue>,
}

pub fn search_notes<R: NoteRepository>(repo: &R, query: &str) -> Result<SearchResults, CoreError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(CoreError::EmptySearchQuery);
    }

    let collection = repo.list_notes()?;
    let matcher = SkimMatcherV2::default().ignore_case();
    let mut matches = collection
        .notes
        .into_iter()
        .filter_map(|note| {
            let score = search_score(&matcher, query, &note.slug, &note.title, &note.tags)?;
            Some(SearchMatch {
                slug: note.slug,
                title: note.title,
                tags: note.tags,
                score,
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.slug.cmp(&right.slug))
    });

    Ok(SearchResults {
        matches,
        issues: collection.issues,
    })
}

fn search_score(
    matcher: &SkimMatcherV2,
    query: &str,
    slug: &str,
    title: &str,
    tags: &[String],
) -> Option<i64> {
    std::iter::once(slug)
        .chain(std::iter::once(title))
        .chain(tags.iter().map(String::as_str))
        .filter_map(|field| field_score(matcher, query, field))
        .max()
}

fn field_score(matcher: &SkimMatcherV2, query: &str, field: &str) -> Option<i64> {
    let fuzzy_score = matcher.fuzzy_match(field, query)?;

    if field.eq_ignore_ascii_case(query) {
        Some(fuzzy_score + 10_000)
    } else if field
        .to_ascii_lowercase()
        .starts_with(&query.to_ascii_lowercase())
    {
        Some(fuzzy_score + 5_000)
    } else {
        Some(fuzzy_score)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::errors::CoreError;
    use crate::core::repository::NoteLoadIssue;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::search_notes;

    #[test]
    fn search_matches_slug_title_and_tags_case_insensitively() {
        let repo = InMemoryNoteRepository::default();
        let mut ownership = note("rust-ownership");
        ownership.title = "Understanding Ownership".to_string();
        ownership.tags = vec!["Rust".to_string(), "learning".to_string()];
        repo.insert(ownership);

        for query in ["rust-ownership", "OWNERSHIP", "rust", "lrn"] {
            let results = search_notes(&repo, query).expect("search should succeed");
            assert_eq!(results.matches.len(), 1, "query: {query}");
            assert_eq!(results.matches[0].slug, "rust-ownership");
        }
    }

    #[test]
    fn exact_match_ranks_above_fuzzy_match() {
        let repo = InMemoryNoteRepository::default();
        let mut exact = note("rust");
        exact.title = "Rust".to_string();
        repo.insert(exact);

        let mut fuzzy = note("rusty-notes");
        fuzzy.title = "Rusty Notes".to_string();
        repo.insert(fuzzy);

        let results = search_notes(&repo, "rust").unwrap();

        assert_eq!(results.matches[0].slug, "rust");
        assert!(results.matches[0].score > results.matches[1].score);
    }

    #[test]
    fn equal_scores_are_ordered_by_slug() {
        let repo = InMemoryNoteRepository::default();
        for slug in ["charlie", "alpha", "bravo"] {
            let mut candidate = note(slug);
            candidate.title = "Shared Search Title".to_string();
            repo.insert(candidate);
        }

        let results = search_notes(&repo, "Shared Search Title").unwrap();
        let slugs = results
            .matches
            .into_iter()
            .map(|result| result.slug)
            .collect::<Vec<_>>();

        assert_eq!(slugs, vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn search_returns_empty_matches_for_no_match() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("garden"));

        let results = search_notes(&repo, "zzzzzz").unwrap();

        assert!(results.matches.is_empty());
    }

    #[test]
    fn search_rejects_empty_or_whitespace_query_before_listing() {
        let repo = InMemoryNoteRepository::default();

        for query in ["", "   ", "\t\n"] {
            let error = search_notes(&repo, query).unwrap_err();
            assert!(matches!(error, CoreError::EmptySearchQuery));
        }
    }

    #[test]
    fn search_preserves_unreadable_note_diagnostics() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("readable"));
        repo.add_list_issue(NoteLoadIssue {
            slug: "broken".to_string(),
            message: "broken frontmatter".to_string(),
        });

        let results = search_notes(&repo, "readable").unwrap();

        assert_eq!(results.matches.len(), 1);
        assert_eq!(results.issues.len(), 1);
        assert_eq!(results.issues[0].slug, "broken");
    }
}
