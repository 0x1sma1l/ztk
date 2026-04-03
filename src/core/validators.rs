use std::collections::HashSet;

use crate::core::errors::CoreError;

pub fn slugify(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn validate_slug(slug: &str) -> Result<String, CoreError> {
    let slug = slug.trim();

    if slug.is_empty() {
        return Err(CoreError::InvalidSlug("Slug cannot be empty".to_string()));
    }

    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(CoreError::InvalidSlug(
            "Use ASCII letters, digits, and '-' only".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(CoreError::InvalidSlug(
            "Slug cannot start or end with '-'".to_string(),
        ));
    }

    Ok(slug.to_string())
}

pub fn validate_tags(raw: &str) -> Result<Vec<String>, CoreError> {
    let mut clean_tags = Vec::new();

    for tag in raw.split(',') {
        let tag = tag.trim();

        if tag.is_empty() {
            continue;
        }

        if !tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(CoreError::InvalidTags(format!(
                "Tag '{}' contains invalid characters. Only alphanumeric, `_`, `-` are allowed.",
                tag
            )));
        }

        clean_tags.push(tag.to_string());
    }

    Ok(clean_tags)
}

pub fn dedup_tags(tags: &mut Vec<String>) {
    let mut seen = HashSet::new();
    tags.retain(|tag| seen.insert(tag.to_lowercase()));
}

pub fn has_duplicate_tags(tags: &[String]) -> bool {
    let mut seen = HashSet::new();
    for tag in tags {
        if !seen.insert(tag.to_lowercase()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_edge_cases_are_stable() {
        let cases = [
            ("", ""),
            ("   ", ""),
            ("Hello World", "hello-world"),
            ("Hello---World", "hello-world"),
            ("  Rust & CLI!!  ", "rust-cli"),
            ("___", ""),
            ("naïve café", "na-ve-caf"),
        ];

        for (input, expected) in cases {
            assert_eq!(slugify(input), expected, "input: {:?}", input);
        }
    }

    #[test]
    fn slugify_is_deterministic() {
        let input = "  Rust & CLI!!  ";
        let first = slugify(input);
        let second = slugify(input);
        assert_eq!(first, second);
    }

    #[test]
    fn validate_tags_accepts_valid_tags() {
        let result = validate_tags("rust, zet_notes, cli-tool").unwrap();
        assert_eq!(result, vec!["rust", "zet_notes", "cli-tool"]);
    }

    #[test]
    fn validate_tags_rejects_invalid_tags() {
        let result = validate_tags("rust, zet_notes, cli-tool!");
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_tag_detection_and_dedup_are_case_insensitive() {
        let tags = vec!["rust".to_string(), "Rust".to_string()];
        assert!(has_duplicate_tags(&tags));

        let mut tags_to_dedup = tags.clone();
        dedup_tags(&mut tags_to_dedup);

        assert_eq!(tags_to_dedup, vec!["rust".to_string()]);
    }
}
