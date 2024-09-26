use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TodoComment {
    pub line: usize,
    pub comment: String,
    pub delimiters: Vec<Delimiter>,
}

#[derive(Serialize, Debug)]
pub struct Delimiter {
    pub delimiter_type: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub enum TodoCommentResult {
    Valid(TodoComment),
    Invalid { line: usize, full_text: String },
}
