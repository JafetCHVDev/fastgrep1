use serde::Serialize;


#[derive(Serialize, Debug, Clone)]
pub struct MatchEntry {
pub file: String,
pub line: usize,
pub text: String,
}


#[derive(Serialize, Debug)]
pub struct Report {
pub pattern: String,
pub total_matches: usize,
pub matches: Vec<MatchEntry>,
}