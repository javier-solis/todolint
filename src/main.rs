use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use utils::{print_todo_result, print_todo_result_json};
mod types;
use types::{
    AnalysisResult, BlameInfo, CaptureGroupNames, CommentMarker, Delimiter, DirectoryAnalysis,
    FileAnalysis, FileMetadata, InvalidContent, InvalidTodoComment, TodoCommentResult,
    ValidContent, ValidTodoComment,
};
mod utils;
use std::path::Path;
use walkdir::WalkDir;

extern crate chrono;

fn main() -> Result<()> {
    Ok(())
}
fn analyze_path(path: &Path) -> Result<AnalysisResult> {
    if path.is_dir() {
        Ok(AnalysisResult::Directory(analyze_dir(path)))
    } else if path.is_file() {
        Ok(AnalysisResult::File(analyze_file(path.to_str().unwrap())?))
    } else {
        Err(anyhow::anyhow!("Path is neither a file nor a directory"))
    }
}

fn analyze_dir(dir: &Path) -> DirectoryAnalysis {
    let file_analyses: Vec<FileAnalysis> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            analyze_file(path.to_str()?).ok()
        })
        .collect();

    let total_files_scanned = file_analyses.len();

    DirectoryAnalysis {
        total_files_scanned,
        last_scan_on: Utc::now(),
        file_analyses,
    }
}

fn analyze_file(filename: &str) -> Result<FileAnalysis> {
    let file = File::open(filename).context("Failed to open file")?;
    let metadata = file.metadata().context("Failed to get file metadata")?;
    let reader = BufReader::new(file);

    let mut file_analysis = FileAnalysis {
        metadata: FileMetadata {
            filepath: filename.to_string(),
            last_modified: metadata.modified()?.into(),
        },
        valids: Vec::new(),
        invalids: Vec::new(),
    };

    for (line_number, line) in reader.lines().enumerate() {
        let line = line.context("Failed to read line")?;
        let processed_line = process_line(&line, line_number)?;

        match processed_line {
            TodoCommentResult::Valid(comment) => {
                file_analysis.valids.push(comment);
            }
            TodoCommentResult::Invalid(comment) => {
                file_analysis.invalids.push(comment);
            }
            _ => {}
        }
    }

    Ok(file_analysis)
}

fn process_line(line: &str, line_number: usize) -> Result<TodoCommentResult> {
    let validation_regex = create_validation_regex(CommentMarker::Todo)
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

    if validate_todo(marker_content).unwrap_or(false) {
        let mut delimiters = Vec::new();
        let delimiter_types = [
            ("()", "parens"),
            ("{}", "braces"),
            ("[]", "brackets"),
            ("<>", "angles"),
        ];

        for (delim, delim_type) in delimiter_types.iter() {
            if let Some(content) = extract_delimiter_content(delim, marker_content) {
                delimiters.push(Delimiter {
                    delimiter_type: delim_type.to_string(),
                    content,
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

/// Validates the contents of a todo (what's between 'todo' and ':'). Returns true if valid,
/// false otherwise.
///
/// Definition of a "valid" todo comment:
/// * There is only 0 or 1 occurrence of each delimiter type:
///   * Types: parentheses, braces, brackets, and angle brackets.
///   * Only characters matching the standard word character class (\w) are allowed between
///     matching delimiter characters.
/// * The order of the delimiters doesn't matter.
///

fn validate_todo(todo_content: &str) -> Result<bool> {
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

        if let Some(captures) = regex.captures(todo_content) {
            let value = captures
                .name(name)
                .ok_or_else(|| anyhow::anyhow!("Failed to get capture group"))?
                .as_str();

            println!("Captured {} content: {}", name, value);

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

/// Extracts the content between the specified delimiter characters in a given line of text.
///
/// This function assumes that the input `line` has already been validated to contain a valid
/// todo comment: the delimiter content is guaranteed to not be empty and only has standard word
/// characters.
///
/// # Arguments
/// * `delimiter` - The delimiter characters to use for extracting the content, e.g. `"<>"` or `"()"`.
/// * `line` - The line of text to extract the content from.
///
/// # Returns
/// An `Option<String>` containing the extracted content, or `None` if the delimiter could not be found.
fn extract_delimiter_content(delimiter: &str, line: &str) -> Option<String> {
    let pattern = if delimiter == "<>" {
        r"<(.*?)>".to_string()
    } else {
        let (open_delim, close_delim) = (delimiter.chars().next()?, delimiter.chars().last()?);
        format!(r"\{}(.*?)\{}", open_delim, close_delim)
    };

    let re = Regex::new(&pattern).ok()?;

    re.captures(line)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_analyze_dir() -> Result<()> {
        let test_dir = Path::new("test");
        let result = analyze_dir(test_dir);

        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);

        Ok(())
    }

    #[test]
    fn test_analyze_file_valid() -> Result<()> {
        let filename = "test/valid.txt";
        let analysis = analyze_file(filename)?;

        assert_eq!(analysis.invalids.len(), 0, "Expected no invalid todos");
        Ok(())
    }

    #[test]
    fn test_analyze_file_invalid() -> Result<()> {
        let filename = "test/invalid.txt";
        let analysis = analyze_file(filename)?;

        assert_eq!(analysis.valids.len(), 0, "Expected no valid todos");
        Ok(())
    }

    fn read_test_file(filename: &str) -> Vec<String> {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        reader.lines().map(|l| l.unwrap()).collect()
    }

    #[derive(Debug, PartialEq)]
    pub enum TodoValidity {
        Valid,
        Invalid,
        NotApplicable,
    }

    #[rstest]
    #[case::valid(read_test_file("test/valid.txt"), TodoValidity::Valid)]
    #[case::invalid(read_test_file("test/invalid.txt"), TodoValidity::Invalid)]
    #[case::na(read_test_file("test/na.txt"), TodoValidity::NotApplicable)]
    fn test_process_line(#[case] lines: Vec<String>, #[case] validity: TodoValidity) {
        for (index, line) in lines.iter().enumerate() {
            let result = process_line(line, index);

            print_todo_result(&result);

            match validity {
                TodoValidity::Valid => {
                    assert!(
                        matches!(result, Ok(TodoCommentResult::Valid(_))),
                        "Expected Valid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::Invalid => {
                    assert!(
                        matches!(result, Ok(TodoCommentResult::Invalid(_))),
                        "Expected Invalid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::NotApplicable => {
                    assert!(
                        matches!(result, Ok(TodoCommentResult::NotApplicable)),
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
    #[case("{}", "hello {world}", Some("world".to_string()))]
    #[case("()", "123 (456)", Some("456".to_string()))]
    #[case("[]", "[brackets]", Some("brackets".to_string()))]
    #[case("<>", "angle <brackets>", Some("brackets".to_string()))]
    #[case("{}", "no braces", None)]
    #[case("()", "mismatched (parenthesis]", None)]
    fn test_extract_delimiter_content(
        #[case] delimiter: &str,
        #[case] line: &str,
        #[case] expected: Option<String>,
    ) {
        let result = extract_delimiter_content(delimiter, line);
        assert_eq!(result, expected);
    }
}
