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
