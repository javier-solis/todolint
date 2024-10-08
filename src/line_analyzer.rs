use anyhow::Result;
use git2::Repository;
use regex::Regex;

use crate::types::{CommentMarker, Delimiter, TodoCommentResult};

struct GitBlameContext {
    repo: Repository, // todo: add an option to specify specific branch?
}

pub struct LineAnalyzer {
    git_blame_context: Option<GitBlameContext>,
    validation_regex: Regex,
}

impl LineAnalyzer {
    pub fn new() -> Result<Self> {
        todo!()
    }

    fn process(&self, line: &str, line_number: usize) -> Result<TodoCommentResult> {
        todo!()
    }

    fn validate(marker_content: &str) -> Result<bool> {
        todo!()
    }

    fn extract_delimiter_content<'a>(
        delimiter: &Delimiter,
        line: &'a str,
    ) -> Result<Option<&'a str>> {
        todo!()
    }

    fn create_validation_regex(marker: CommentMarker) -> Result<Regex> {
        todo!()
    }
}
