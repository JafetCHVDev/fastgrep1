use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchEntry {
    pub file: String,
    pub line: usize,
    pub matched_text: String, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Report {
    pub pattern: String,
    pub total_matches: usize,
    pub matches: Vec<MatchEntry>,
}
