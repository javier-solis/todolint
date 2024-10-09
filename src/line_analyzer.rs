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
    validation_regex: Regex,
}

impl LineAnalyzer {
    pub fn new() -> Result<Self> {
        // todo: add CommentMarker as a param?
        Ok(LineAnalyzer {
            validation_regex: Self::create_validation_regex(CommentMarker::Todo)?,
        })
    }

    pub fn process(&self, line: &str, line_number: usize) -> Result<Option<TodoCommentResult>> {
        let validation_regex = &self.validation_regex;

        let general_cap = match validation_regex.captures(line) {
            Some(cap) => cap,
            None => return Ok(None),
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
                        delimiter_type: delimiter,
                        content: content.to_string(),
                    });
                }
            }

            Ok(Some(TodoCommentResult::Valid(ValidTodoComment {
                line: line_number,
                line_info: ValidContent {
                    comment: comment_content.to_string(),
                    delimiters,
                },
            })))
        } else {
            Ok(Some(TodoCommentResult::Invalid(InvalidTodoComment {
                line: line_number,
                line_info: InvalidContent {
                    full_text: general_cap[0].to_string(),
                },
            })))
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    #[derive(Debug, PartialEq)]
    pub enum TodoValidity {
        Valid,
        Invalid,
        NotApplicable,
    }

    fn read_test_file(filename: &str) -> Vec<String> {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        reader.lines().map(|l| l.unwrap()).collect()
    }

    #[rstest]
    #[case::valid(read_test_file("test/valid.txt"), TodoValidity::Valid)]
    #[case::invalid(read_test_file("test/invalid.txt"), TodoValidity::Invalid)]
    #[case::na(read_test_file("test/na.txt"), TodoValidity::NotApplicable)]
    fn test_process_line(#[case] lines: Vec<String>, #[case] validity: TodoValidity) {
        for (index, line) in lines.iter().enumerate() {
            let line_analyzer_obj = LineAnalyzer::new().unwrap();
            let result = line_analyzer_obj.process(line, index);

            match validity {
                TodoValidity::Valid => {
                    assert!(
                        matches!(result, Ok(Some(TodoCommentResult::Valid(_)))),
                        "Expected Valid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::Invalid => {
                    assert!(
                        matches!(result, Ok(Some(TodoCommentResult::Invalid(_)))),
                        "Expected Invalid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::NotApplicable => {
                    assert!(
                        matches!(result, Ok(None)),
                        "Expected n/a but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
            }
        }
    }

    #[rstest]
    #[case(Delimiter::Braces, "hello {world}", Ok(Some("world")))]
    #[case(Delimiter::Parentheses, "123 (456)", Ok(Some("456")))]
    #[case(Delimiter::Brackets, "[brackets]", Ok(Some("brackets")))]
    #[case(Delimiter::Angles, "angle <brackets>", Ok(Some("brackets")))]
    #[case(Delimiter::Braces, "no braces", Ok(None))]
    #[case(Delimiter::Parentheses, "mismatched (parenthesis]", Ok(None))]
    #[case(Delimiter::Braces, "no braces", Ok(None))]
    fn test_extract_delimiter_content(
        #[case] delimiter: Delimiter,
        #[case] line: &str,
        #[case] expected: Result<Option<&str>>,
    ) {
        let result = LineAnalyzer::extract_delimiter_content(&delimiter, line);
        match (&result, &expected) {
            (Ok(actual), Ok(expected)) => assert_eq!(actual, expected),
            (Err(_), Err(_)) => assert!(true), // both are errors, test passes
            _ => panic!("Result {:?} does not match expected {:?}", result, expected),
        }
    }

    #[rstest]
    #[case(CommentMarker::Todo, "// todo: valid", true)]
    #[case(CommentMarker::Todo, "//todo      : valid", true)]
    #[case(CommentMarker::Todo, "//         todo      : valid", true)]
    #[case(CommentMarker::Todo, "// TODO: invalid", false)]
    #[case(CommentMarker::Todo, "// todo this is missing a colon", false)]
    #[case(CommentMarker::Todo, "/ todo: missing a slash", false)]
    #[case(CommentMarker::Todo, "// todo:", false)] // (missing comment content)
    #[case(CommentMarker::Todo, "// todox: This should not match", false)]
    fn test_create_validation_regex(
        #[case] marker: CommentMarker,
        #[case] input: &str,
        #[case] should_match: bool,
    ) {
        let regex = LineAnalyzer::create_validation_regex(marker).unwrap();
        assert_eq!(
            regex.is_match(input),
            should_match,
            "Regex match failed for input '{}'",
            input,
        );
    }
}
