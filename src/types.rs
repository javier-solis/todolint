pub struct TodoComment {
    pub line: usize,
    pub comment: String,
    pub delimiters: Vec<Delimiter>,
}

pub struct Delimiter {
    pub delimiter_type: String,
    pub content: String,
}

pub enum TodoCommentResult {
    Valid(TodoComment),
    Invalid { line: usize, full_text: String },
}
