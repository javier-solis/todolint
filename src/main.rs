use regex::Regex;
use std::fs;

fn main() {
    let filename = "src/main.rs";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    // Regex to match any line that contains `// todo ... :` and capture the content
    let todo_prefix = r"//\s*todo\s*";
    let todo_content = r"(?<todo_content>.*?)";
    let colon_separator = r"\s*:\s*";
    let comment_content = r"(?<comment_content>.*)";

    let general_todo_re = Regex::new(&format!(
        r"{}{}{}{}",
        todo_prefix, todo_content, colon_separator, comment_content
    ))
    .unwrap();

    // Regex to match correctly formatted 'todo' comments
    let keyword_pattern = r"[a-zA-Z0-9_-]+";
    let parens_pattern = format!(r"\((?<parens>{})\)", keyword_pattern);
    let braces_pattern = format!(r"\{{(?<braces>{})\}}", keyword_pattern);
    let brackets_pattern = format!(r"\[(?<brackets>{})\]", keyword_pattern);
    let angles_pattern = format!(r"<(?<angles>{})>", keyword_pattern);
    let delimiter_pattern = format!(
        r"(?:{}|{}|{}|{})",
        parens_pattern, braces_pattern, brackets_pattern, angles_pattern
    );
    let specific_todo_re = Regex::new(&format!(r"^{}{{0,3}}$", delimiter_pattern)).unwrap();

    // Parse through file to match against the general regex
    for (line_number, line) in contents.lines().enumerate() {
        if let Some(general_cap) = general_todo_re.captures(line) {
            let todo_comment = &general_cap[0];
            let todo_content = &general_cap["todo_content"];
            let comment_content = &general_cap["comment_content"];

            println!("'todo' on line {}:", line_number + 1);
            println!("\tFull text: {}", todo_comment);
            println!("\tTodo content: {}", todo_content);
            println!("\tComment content: {}", comment_content);

            // Match against the specific regex
            if specific_todo_re.is_match(todo_content) {
                println!("\tIs Valid: True");
            } else {
                println!("\tIs Valid: False");
            }
            println!("\n");
        }
    }
}
