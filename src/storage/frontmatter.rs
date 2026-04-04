use serde::{Deserialize, Serialize};

use crate::core::errors::CoreError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn parse_frontmatter_and_body(content: &str) -> Result<(Frontmatter, String), CoreError> {
    let sections: Vec<&str> = content.splitn(3, "---").collect();

    if sections.len() < 3 {
        return Err(CoreError::EmptyFrontmatter);
    }

    let yaml_block = sections[1];
    let frontmatter: Frontmatter =
        serde_yaml::from_str(yaml_block).map_err(CoreError::FrontmatterParse)?;

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
        };
        let body = "# Heading\n\nBody content.\n";

        let content = build_note_content(&fm, body).expect("build should succeed");
        let (parsed_fm, parsed_body) =
            parse_frontmatter_and_body(&content).expect("parse should succeed");

        assert_eq!(parsed_fm.title, fm.title);
        assert_eq!(parsed_fm.date, fm.date);
        assert_eq!(parsed_fm.tags, fm.tags);
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
}
