use serde_json;

use crate::types::{Delimiter, TodoComment, TodoCommentResult};

pub fn print_todo_result_json(result: &Option<TodoCommentResult>) {
    if let Some(todo_result) = result {
        let json = serde_json::to_string_pretty(todo_result).unwrap();
        println!("{}\n", json);
    }
}

pub fn print_todo_result(result: &Option<TodoCommentResult>) {
    if let Some(todo_result) = result {
        match todo_result {
            TodoCommentResult::Valid(todo_comment) => {
                println!("'todo' on line {}:", todo_comment.line);
                println!("\tIs Valid: True");
                println!("\tComment content: {}", todo_comment.comment);
                println!(
                    "\tDelimiters Found: {:?}",
                    todo_comment
                        .delimiters
                        .iter()
                        .map(|d| d.delimiter_type.as_str())
                        .collect::<Vec<_>>()
                );
                for delimiter in &todo_comment.delimiters {
                    println!(
                        "\tContents of {}: {}",
                        delimiter.delimiter_type, delimiter.content
                    );
                }
            }
            TodoCommentResult::Invalid { line, full_text } => {
                println!("'todo' on line {}:", line);
                println!("\tIs Valid: False");
                println!("\tFull text: {}", full_text);
            }
        }
        println!("\n");
    }
}
