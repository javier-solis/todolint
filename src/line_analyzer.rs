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
        let validation_regex = Self::create_validation_regex(CommentMarker::Todo)
            .context("Failed to create validation regex")?;

        let general_cap = match validation_regex.captures(line) {
            Some(cap) => cap,
            None => return Ok(TodoCommentResult::NotApplicable),
        };

        let marker_content = general_cap
            .name(CaptureGroupNames::MarkerContent.as_ref())
            .map(|m| m.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing '{}' capture group",
                    CaptureGroupNames::MarkerContent
                )
            })?;

        let comment_content = general_cap
            .name(CaptureGroupNames::CommentContent.as_ref())
            .map(|m| m.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Missing '{}' capture group",
                    CaptureGroupNames::CommentContent
                )
            })?;

        let line_number = line_number + 1; // for human readable purposes

        if Self::validate(marker_content).unwrap_or(false) {
            let mut delimiters = Vec::new();

            for delimiter in Delimiter::iter() {
                if let Ok(Some(content)) =
                    Self::extract_delimiter_content(&delimiter, marker_content)
                {
                    delimiters.push(DelimiterContent {
                        delimiter_type: delimiter.get_name().to_string(),
                        content: content.to_string(),
                    });
                }
            }

            Ok(TodoCommentResult::Valid(ValidTodoComment {
                line: line_number,
                line_info: ValidContent {
                    comment: comment_content.to_string(),
                    delimiters,
                },
            }))
        } else {
            Ok(TodoCommentResult::Invalid(InvalidTodoComment {
                line: line_number,
                line_info: InvalidContent {
                    full_text: general_cap[0].to_string(),
                },
            }))
        }
    }

    /// Validates the contents of a todo (what's between 'todo' and ':'). Returns true if valid,
    /// false otherwise.
    fn validate(marker_content: &str) -> Result<bool> {
        let keyword_pattern = r".*?";
        let delimiters = [
            (r"\((?<parens>{})\)", "parens"),
            (r"\{(?<braces>{})\}", "braces"),
            (r"\[(?<brackets>{})\]", "brackets"),
            (r"<(?<angles>{})>", "angles"),
        ];

        let mut found_delimiters = Vec::new();

        for (pattern, name) in &delimiters {
            let regex = Regex::new(&format!(r"^{}$", pattern.replace("{}", keyword_pattern)))
                .with_context(|| format!("Failed to create regex for {}", name))?;

            if let Some(captures) = regex.captures(marker_content) {
                let value = captures
                    .name(name)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get capture group"))?
                    .as_str();

                // todo: simplify or use a variable/helper-function?
                if value.is_empty() || Regex::new(r"[^\w]").unwrap().is_match(value) {
                    return Ok(false);
                }

                if found_delimiters.contains(name) {
                    return Ok(false); // Duplicate delimiter found
                }
                found_delimiters.push(name);
            }
        }

        Ok(found_delimiters.len() <= 4)
    }

    /// Extracts content between specified delimiter characters in a given line of text.
    /// Returns None if no valid delimited content is found.
    fn extract_delimiter_content<'a>(
        delimiter: &Delimiter,
        line: &'a str,
    ) -> Result<Option<&'a str>> {
        let chars = delimiter.get_chars();
        let (open_delim, close_delim) = chars.to_tuple();

        let pattern = if *delimiter == Delimiter::Angles {
            format!(r"{}(.*?){}", open_delim, close_delim)
        } else {
            format!(r"\{}(.*?)\{}", open_delim, close_delim)
        };

        let re = Regex::new(&pattern).context("Failed to create regex")?;

        Ok(re
            .captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str()))
    }

    fn create_validation_regex(marker: CommentMarker) -> Result<Regex> {
        let prefix = format!(r"//\s*{}\s*", marker);
        let marker_content = format!(r"(?<{}>.*?)", CaptureGroupNames::MarkerContent);
        let colon_separator = r"\s*:\s*";
        let comment_content = format!(r"(?<{}>.*)", CaptureGroupNames::CommentContent);

        Regex::new(&format!(
            r"{}{}{}{}",
            prefix, marker_content, colon_separator, comment_content
        ))
        .context("Failed to create validation regex")
    }
}
