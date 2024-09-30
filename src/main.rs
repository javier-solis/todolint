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
    Delimiter, DirectoryAnalysis, FileAnalysis, FileMetadata, InvalidTodoComment, TodoComment,
    TodoCommentResult,
};
mod utils;
use std::path::Path;
use walkdir::WalkDir;

extern crate chrono;

fn main() -> Result<()> {
    Ok(())
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

    let general_todo_re = create_general_todo_regex()?;

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
        let processed_line = process_line(&line, line_number, &general_todo_re);

        match processed_line {
            Some(TodoCommentResult::Valid(comment)) => {
                file_analysis.valids.push(comment);
            }
            Some(TodoCommentResult::Invalid(comment)) => {
                file_analysis.invalids.push(comment);
            }
            None => {}
        }
    }

    Ok(file_analysis)
}

fn process_line(line: &str, line_number: usize) -> Option<TodoCommentResult> {
    let general_todo_re = create_general_todo_regex().unwrap();

    let general_cap = match general_todo_re.captures(line) {
        Some(cap) => cap,
        None => return None,
    };

    let todo_content = &general_cap["todo_content"];
    let comment_content = &general_cap["comment_content"];

    if validate_todo(todo_content).unwrap_or(false) {
        let mut delimiters = Vec::new();
        let delimiter_types = [
            ("()", "parens"),
            ("{}", "braces"),
            ("[]", "brackets"),
            ("<>", "angles"),
        ];

        for (delim, delim_type) in delimiter_types.iter() {
            if let Some(content) = extract_delimiter_content(delim, todo_content) {
                delimiters.push(Delimiter {
                    delimiter_type: delim_type.to_string(),
                    content,
                });
            }
        }

        Some(TodoCommentResult::Valid(TodoComment {
            line: line_number + 1,
            comment: comment_content.to_string(),
            delimiters,
        }))
    } else {
        Some(TodoCommentResult::Invalid(InvalidTodoComment {
            line: line_number + 1,
            full_text: general_cap[0].to_string(),
        }))
    }
}

fn create_general_todo_regex() -> Result<Regex> {
    let todo_prefix = r"//\s*todo\s*";
    let todo_content = r"(?<todo_content>.*?)";
    let colon_separator = r"\s*:\s*";
    let comment_content = r"(?<comment_content>.*)";

    Regex::new(&format!(
        r"{}{}{}{}",
        todo_prefix, todo_content, colon_separator, comment_content
    ))
    .context("Failed to create general todo regex")
}

/// Definition of a "valid" todo comment:
/// * There is only 0 or 1 occurrence of each delimiter type:
///   * Types: parentheses, braces, brackets, and angle brackets.
///    * Only characters matching the standard word character class (\w) are allowed between the delimiters
/// * The order of the delimiters doesn't matter.
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
                        matches!(result, Some(TodoCommentResult::Valid(_))),
                        "Expected Valid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::Invalid => {
                    assert!(
                        matches!(result, Some(TodoCommentResult::Invalid(_))),
                        "Expected Invalid but got {:?} for line {}: {}",
                        result,
                        index + 1,
                        line
                    );
                }
                TodoValidity::NotApplicable => {
                    assert!(
                        result.is_none(),
                        "Expected None but got {:?} for line {}: {}",
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
