use anyhow::{Context, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use utils::{print_todo_result, print_todo_result_json};
mod types;
use types::{Delimiter, TodoComment, TodoCommentResult};
mod utils;

fn main() -> Result<()> {
    let filename = "src/main.rs";
    analyze_file(filename)
}

fn analyze_file(filename: &str) -> Result<()> {
    let file = File::open(filename).context("Failed to open file")?;
    let reader = BufReader::new(file);

    let general_todo_re = create_general_todo_regex()?;
    let specific_todo_re = create_specific_todo_regex()?;

    for (line_number, line) in reader.lines().enumerate() {
        let line = line.context("Failed to read line")?;
        let processed_line = process_line(&line, line_number, &general_todo_re, &specific_todo_re);
        print_todo_result(&processed_line);
        print_todo_result_json(&processed_line);
    }

    Ok(())
}

fn process_line(
    line: &str,
    line_number: usize,
    general_re: &Regex,
    specific_re: &Regex,
) -> Option<TodoCommentResult> {
    if let Some(general_cap) = general_re.captures(line) {
        let todo_content = &general_cap["todo_content"];
        let comment_content = &general_cap["comment_content"];

        if specific_re.is_match(todo_content) {
            let mut delimiters = Vec::new();

            if let Some(specific_cap) = specific_re.captures(todo_content) {
                let delimiter_types = ["parens", "braces", "brackets", "angles"];
                let matched_delimiters: Vec<(&str, &str)> = delimiter_types
                    .iter()
                    .filter_map(|&delimiter_type| {
                        specific_cap
                            .name(delimiter_type)
                            .map(|matched_content| (delimiter_type, matched_content.as_str()))
                    })
                    .collect();

                for (delimiter_type, delimiter_content) in matched_delimiters {
                    delimiters.push(Delimiter {
                        delimiter_type: delimiter_type.to_string(),
                        content: delimiter_content.to_string(),
                    });
                }
            }

            Some(TodoCommentResult::Valid(TodoComment {
                line: line_number + 1,
                comment: comment_content.to_string(),
                delimiters,
            }))
        } else {
            Some(TodoCommentResult::Invalid {
                line: line_number + 1,
                full_text: general_cap[0].to_string(),
            })
        }
    } else {
        None
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

fn create_specific_todo_regex() -> Result<Regex> {
    let keyword_pattern = r"[a-zA-Z0-9_-]+";

    let parens_pattern = format!(r"\((?<parens>{})\)", keyword_pattern);
    let braces_pattern = format!(r"\{{(?<braces>{})\}}", keyword_pattern);
    let brackets_pattern = format!(r"\[(?<brackets>{})\]", keyword_pattern);
    let angles_pattern = format!(r"<(?<angles>{})>", keyword_pattern);

    let delimiter_pattern = format!(
        r"(?:{}|{}|{}|{})",
        parens_pattern, braces_pattern, brackets_pattern, angles_pattern
    );

    Regex::new(&format!(r"^{}{{0,4}}$", delimiter_pattern))
        .context("Failed to create specific todo regex")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_file() {
        let test_file = "test/valid.txt";
        let result = analyze_file(test_file);
        assert!(result.is_ok());
    }
}
