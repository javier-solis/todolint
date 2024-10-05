use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use email_address::EmailAddress;
use git2::{BlameOptions, Repository};
use std::{path::Path, str::FromStr};

use serde_json;

use crate::types::{BlameInfo, TodoCommentResult};

pub fn print_json<T: serde::Serialize>(item: &T) {
    let json = serde_json::to_string_pretty(item).unwrap();
    println!("{}\n", json);
}

pub fn print_todo_result(result: &Result<TodoCommentResult>) {
    if let Ok(todo_result) = result {
        match todo_result {
            TodoCommentResult::Valid(comment) => {
                println!("'todo' on line {}:", comment.line);
                println!("\tIs Valid: True");
                println!("\tComment content: {}", comment.line_info.comment);
                println!(
                    "\tDelimiters Found: {:?}",
                    comment
                        .line_info
                        .delimiters
                        .iter()
                        .map(|d| d.delimiter_type.as_str())
                        .collect::<Vec<_>>()
                );
                for delimiter in &comment.line_info.delimiters {
                    println!(
                        "\tContents of {}: {}",
                        delimiter.delimiter_type, delimiter.content
                    );
                }
            }
            TodoCommentResult::Invalid(comment) => {
                println!("'todo' on line {}:", comment.line);
                println!("\tIs Valid: False");
                println!("\tFull text: {}", comment.line_info.full_text);
            }
            _ => {}
        }
        println!("\n");
    }
}

/// Retrieve git blame information (email and timestamp) for a specific line in a file within a
/// repository.
pub fn get_blame_info(file_path: &Path, line_number: usize) -> Result<BlameInfo> {
    let repo = Repository::discover(file_path)
        .with_context(|| format!("Failed to discover repository for file {:?}", file_path))?;

    // Get blame information for the specified file and line, then retrieve the corresponding commit
    let mut blame_opts = BlameOptions::new();
    let blame = repo
        .blame_file(file_path, Some(&mut blame_opts))
        .with_context(|| format!("Failed to get blame for file {:?}", file_path))?;
    let hunk = blame
        .get_line(line_number)
        .with_context(|| format!("No blame information found for line {}", line_number))?;
    let commit_id = hunk.final_commit_id();
    let commit = repo
        .find_commit(commit_id)
        .with_context(|| format!("Failed to find commit {}", commit_id))?;

    // From commit, extract author email and timestamp
    let author_email = commit
        .author()
        .email()
        .and_then(|email| EmailAddress::from_str(email).ok())
        .unwrap_or_else(|| EmailAddress::new_unchecked("unknown@example.com"));
    let timestamp = Utc
        .timestamp_opt(commit.time().seconds(), 0)
        .single()
        .with_context(|| format!("Invalid timestamp: {}", commit.time().seconds()))?;

    Ok(BlameInfo {
        email: author_email,
        timestamp,
    })
}
