use anyhow::{Context, Result};
use git2::Repository;
use regex::Regex;
use strum::IntoEnumIterator;

use crate::types::{
    CaptureGroupNames, CommentMarker, Delimiter, DelimiterContent, InvalidContent,
    InvalidTodoComment, TodoCommentResult, ValidContent, ValidTodoComment,
};

struct GitBlameContext {
    repo: Repository, // todo: add an option to specify specific branch?
}

pub struct LineAnalyzer {
    // todo: use these attributes
    // git_blame_context: Option<GitBlameContext>,
    // validation_regex: Regex,
}

impl LineAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(LineAnalyzer {})
    }

    pub fn process(&self, line: &str, line_number: usize) -> Result<TodoCommentResult> {
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
