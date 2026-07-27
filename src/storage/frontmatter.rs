use serde::{Deserialize, Serialize};

use crate::core::errors::CoreError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct StoredFrontmatter {
    title: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

pub fn parse_frontmatter_and_body(content: &str) -> Result<(Frontmatter, String), CoreError> {
    let sections: Vec<&str> = content.splitn(3, "---").collect();

    if sections.len() < 3 {
        return Err(CoreError::EmptyFrontmatter);
    }

    let yaml_block = sections[1];
    let stored: StoredFrontmatter =
        serde_yaml::from_str(yaml_block).map_err(CoreError::FrontmatterParse)?;
    let updated_at = stored.updated_at.unwrap_or_else(|| stored.date.clone());
    let frontmatter = Frontmatter {
        title: stored.title,
        date: stored.date,
        tags: stored.tags,
        updated_at,
    };

    Ok((frontmatter, sections[2].to_string()))
}

pub fn build_note_content(frontmatter: &Frontmatter, body: &str) -> Result<String, CoreError> {
    let yaml = serde_yaml::to_string(frontmatter).map_err(CoreError::FrontmatterSerialize)?;
    Ok(format!(
        "---\n{}---\n\n{}",
        yaml,
        body.trim_start_matches('\n')
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_roundtrip_preserves_data() {
        let fm = Frontmatter {
            title: "Roundtrip Title".to_string(),
            date: "2026-04-04".to_string(),
            tags: vec!["rust".to_string(), "week3".to_string()],
            updated_at: "2026-04-04".to_string(),
        };
        let body = "# Heading\n\nBody content.\n";

        let content = build_note_content(&fm, body).expect("build should succeed");
        let (parsed_fm, parsed_body) =
            parse_frontmatter_and_body(&content).expect("parse should succeed");

        assert_eq!(parsed_fm.title, fm.title);
        assert_eq!(parsed_fm.date, fm.date);
        assert_eq!(parsed_fm.tags, fm.tags);
        assert_eq!(parsed_fm.updated_at, fm.updated_at);
        assert_eq!(parsed_body.trim_start_matches('\n'), body);
    }

    #[test]
    fn parse_frontmatter_returns_error_for_missing_frontmatter() {
        let content = "# No frontmatter here\n\njust body";
        let err = parse_frontmatter_and_body(content).unwrap_err();

        match err {
            CoreError::EmptyFrontmatter => {}
            other => panic!("expected EmptyFrontmatter, got {other:?}"),
        }
    }

    #[test]
    fn parse_frontmatter_returns_error_for_invalid_yaml() {
        let content = "\
    ---
    title: Valid
    date: [not-a-string
    tags: [rust]
    ---

    Body
    ";
        let err = parse_frontmatter_and_body(content).unwrap_err();

        match err {
            CoreError::FrontmatterParse(_) => {}
            other => panic!("expected FrontmatterParse, got {other:?}"),
        }
    }

    #[test]
    fn missing_updated_at_uses_date_as_fallback() {
        let content = "---\ntitle: Legacy Note\ndate: 2026-04-04\ntags: [legacy]\n---\n\nBody\n";

        let (frontmatter, body) =
            parse_frontmatter_and_body(content).expect("legacy note should parse");

        assert_eq!(frontmatter.updated_at, "2026-04-04");
        assert_eq!(body.trim_start_matches('\n'), "Body\n");
    }

    #[test]
    fn explicit_updated_at_is_preserved() {
        let content = "---\ntitle: Updated Note\ndate: 2026-04-04\ntags: []\nupdated_at: 2026-07-27\n---\n\nBody\n";

        let (frontmatter, _) =
            parse_frontmatter_and_body(content).expect("current note should parse");

        assert_eq!(frontmatter.updated_at, "2026-07-27");
    }

    #[test]
    fn explicitly_empty_updated_at_is_not_silently_repaired() {
        let content = "---\ntitle: Empty Update Date\ndate: 2026-04-04\ntags: []\nupdated_at: ''\n---\n\nBody\n";

        let (frontmatter, _) =
            parse_frontmatter_and_body(content).expect("empty string is valid YAML");

        assert_eq!(frontmatter.updated_at, "");
    }

    #[test]
    fn null_updated_at_uses_date_as_fallback() {
        let content = "---\ntitle: Null Update Date\ndate: 2026-04-04\ntags: []\nupdated_at: null\n---\n\nBody\n";

        let (frontmatter, _) =
            parse_frontmatter_and_body(content).expect("null should use compatibility fallback");

        assert_eq!(frontmatter.updated_at, "2026-04-04");
    }
}
