use serde_json;

use crate::types::TodoCommentResult;

pub fn print_json<T: serde::Serialize>(item: &T) {
    let json = serde_json::to_string_pretty(item).unwrap();
    println!("{}\n", json);
}

pub fn print_todo_result(result: &Option<TodoCommentResult>) {
    if let Some(todo_result) = result {
        match todo_result {
            TodoCommentResult::Valid(comment) => {
                println!("'todo' on line {}:", comment.line);
                println!("\tIs Valid: True");
                println!("\tComment content: {}", comment.comment);
                println!(
                    "\tDelimiters Found: {:?}",
                    comment
                        .delimiters
                        .iter()
                        .map(|d| d.delimiter_type.as_str())
                        .collect::<Vec<_>>()
                );
                for delimiter in &comment.delimiters {
                    println!(
                        "\tContents of {}: {}",
                        delimiter.delimiter_type, delimiter.content
                    );
                }
            }
            TodoCommentResult::Invalid(comment) => {
                println!("'todo' on line {}:", comment.line);
                println!("\tIs Valid: False");
                println!("\tFull text: {}", comment.full_text);
            }
        }
        println!("\n");
    }
}
