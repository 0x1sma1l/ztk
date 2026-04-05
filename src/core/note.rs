#[derive(Debug, Clone)]
pub struct Note {
    pub slug: String,
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub updated_at: String,
    pub body: String,
}
