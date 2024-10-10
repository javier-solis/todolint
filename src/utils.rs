use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use email_address::EmailAddress;
use git2::{BlameOptions, Repository};
use std::{path::Path, str::FromStr};

use serde_json;

use crate::line_analyzer_types::{BlameInfo, TodoCommentResult};

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
                        .map(|d| d.delimiter_type.get_name())
                        .collect::<Vec<_>>()
                );
                for delimiter in &comment.line_info.delimiters {
                    println!(
                        "\tContents of {:#?}: {}",
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
